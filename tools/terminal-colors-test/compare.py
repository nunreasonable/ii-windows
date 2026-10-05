#!/usr/bin/env python3
"""Checks terminal_colors_test (the Windows backend's C++ terminal color harmonization) against
ii's scripts/colors/generate_colors_material.py, color for color.

Two sets of cases, both fed to the C++ program and compared hex for hex:
  * end to end: the real script, run the way switchwall.sh runs it (--color, --termscheme,
    --blend_bg_fg, --harmony, --harmonize_threshold, --term_fg_boost) over a matrix of seed colors
    (greys and near greys too), schemes, modes and settings. The material colors it prints
    (primary_paletteKeyColor, surfaceContainerLow, onSurface) are the C++ side's inputs.
  * in process: the script's own harmonize/boost_chroma_tone and terminal loop, pulled out of
    its source, on random inputs (any primary key color, exact greys, random base colors and
    settings), many more than spawning the script would allow.

With --hct it also checks the HCT conversion under all of it, Hct(argb) for every sRGB color,
bit for bit (as CRC-32s of 65536-color blocks).

Run it with the Python ii uses (it needs materialyoucolor and PIL):
  ~/.local/state/quickshell/.venv/bin/python compare.py [--hct] <terminal_colors_test binary>
  ... compare.py [--hct] --write <dir>   only writes <dir>/cases.txt and <dir>/expected.txt
                                         (and expected-hct.txt), for a Windows build of the test
                                         to be diffed against on the target.
"""

import argparse
import ast
import json
import random
import struct
import subprocess
import sys
import zlib
from concurrent.futures import ThreadPoolExecutor
from multiprocessing import Pool
from pathlib import Path
from types import SimpleNamespace

IIW = Path(__file__).resolve().parents[2]
SCRIPT = IIW / "ii/scripts/colors/generate_colors_material.py"
TERMSCHEME = IIW / "ii/scripts/colors/terminal/scheme-base.json"
NAMES = [f"term{i}" for i in range(16)]

SEEDS = [
    "#4A7FD0", "#D04A4A", "#3FA34D", "#E8C547", "#8E44AD", "#FF00FF", "#00897B",
    "#808080", "#7F8082", "#8A8580", "#0A0A0A", "#FAFAFA",
]
SCHEMES = [
    "scheme-tonal-spot", "scheme-vibrant", "scheme-neutral", "scheme-fidelity", "scheme-content",
    "scheme-expressive", "scheme-rainbow", "scheme-fruit-salad", "scheme-monochrome",
]
HARMONY = ["0.6", "0.8", "1.0"]
THRESHOLD = ["100", "35"]
FG_BOOST = ["0.35", "0"]


def case_line(mode, harmony, threshold, fg_boost, monochrome, primary, scl, on_surface, base):
    numbers = " ".join(float(x).hex() for x in (harmony, threshold, fg_boost))
    colors = " ".join(base[name] for name in NAMES)
    return f"{mode} {numbers} {int(monochrome)} {primary} {scl} {on_surface} {colors}"


def end_to_end_cases(script, termscheme):
    base_schemes = json.loads(termscheme.read_text())
    runs = []
    k = 0
    for seed in SEEDS:
        for mode in ("dark", "light"):
            for harmony in HARMONY:
                for threshold in THRESHOLD:
                    for fg_boost in FG_BOOST:
                        runs.append((seed, mode, SCHEMES[k % len(SCHEMES)], harmony, threshold, fg_boost))
                        k += 1
            runs.append((seed, mode, "monochrome", "0.8", "100", "0.35"))

    def run(params):
        seed, mode, scheme, harmony, threshold, fg_boost = params
        out = subprocess.run(
            [sys.executable, str(script), "--color", seed, "--mode", mode, "--scheme", scheme,
             "--termscheme", str(termscheme), "--blend_bg_fg", "--harmony", harmony,
             "--harmonize_threshold", threshold, "--term_fg_boost", fg_boost],
            capture_output=True, text=True, check=True,
        ).stdout
        values = {}
        for line in out.splitlines():
            if line.startswith("$") and ": " in line:
                name, value = line[1:].rstrip(";").split(": ", 1)
                values[name] = value
        line = case_line(
            mode, harmony, threshold, fg_boost, scheme == "monochrome",
            values["primary_paletteKeyColor"], values["surfaceContainerLow"], values["onSurface"],
            base_schemes[mode],
        )
        label = f"{seed} {mode} {scheme} harmony={harmony} threshold={threshold} fgBoost={fg_boost}"
        return label, line, [values[name] for name in NAMES]

    with ThreadPoolExecutor() as pool:
        return list(pool.map(run, runs))


def load_script(script):
    tree = ast.parse(script.read_text(), str(script))
    helpers, loop = [], []
    for node in tree.body:
        if isinstance(node, (ast.Import, ast.ImportFrom)):
            helpers.append(node)
        elif isinstance(node, ast.FunctionDef) and node.name in ("harmonize", "boost_chroma_tone"):
            helpers.append(node)
        elif isinstance(node, ast.Assign) and any(
            isinstance(t, ast.Name) and t.id in ("argb_to_hex", "hex_to_argb") for t in node.targets
        ):
            helpers.append(node)
        elif isinstance(node, ast.If) and "termscheme" in ast.unparse(node.test):
            for inner in node.body:
                if isinstance(inner, ast.For) or (
                    isinstance(inner, ast.Assign) and ast.unparse(inner.targets[0]) == "primary_color_argb"
                ):
                    loop.append(inner)
    if len(helpers) < 4 or len(loop) != 2:
        sys.exit(f"{script}: couldn't find harmonize/boost_chroma_tone/the terminal loop")
    namespace = {}
    exec(compile(ast.Module(body=helpers, type_ignores=[]), str(script), "exec"), namespace)
    return namespace, compile(ast.Module(body=loop, type_ignores=[]), str(script), "exec")


def in_process_cases(script, termscheme, count, seed):
    namespace, loop = load_script(script)
    base_schemes = json.loads(termscheme.read_text())
    rng = random.Random(seed)

    def color():
        kind = rng.random()
        if kind < 0.15:
            g = rng.choice([0, 0xFF, rng.randrange(256)])
            return f"#{g:02X}{g:02X}{g:02X}"
        if kind < 0.35:
            g = rng.randrange(3, 253)
            return "#" + "".join(f"{min(255, max(0, g + rng.randint(-3, 3))):02X}" for _ in range(3))
        return f"#{rng.randrange(1 << 24):06X}"

    cases = []
    for i in range(count):
        mode = rng.choice(["dark", "light"])
        harmony = rng.choice([0.6, 0.8, 1.0, rng.uniform(0, 1)])
        threshold = rng.choice([100.0, 35.0, rng.uniform(0, 180)])
        fg_boost = rng.choice([0.35, 0.0, rng.uniform(0, 0.6)])
        monochrome = rng.random() < 0.05
        primary, scl, on_surface = color(), color(), color()
        base = dict(base_schemes[mode]) if rng.random() < 0.3 else {name: color() for name in NAMES}

        scope = dict(namespace)
        scope.update(
            args=SimpleNamespace(
                scheme="monochrome" if monochrome else "scheme-tonal-spot", blend_bg_fg=True,
                harmony=harmony, harmonize_threshold=threshold, term_fg_boost=fg_boost,
            ),
            material_colors={"primary_paletteKeyColor": primary, "surfaceContainerLow": scl, "onSurface": on_surface},
            term_source_colors=base, term_colors={}, darkmode=mode == "dark",
        )
        exec(loop, scope)

        line = case_line(mode, harmony, threshold, fg_boost, monochrome, primary, scl, on_surface, base)
        label = f"random #{i}: {line[:80]}..."
        cases.append((label, line, [scope["term_colors"][name] for name in NAMES]))
    return cases


def hct_block_crc(block):
    from materialyoucolor.hct import Hct

    crc = 0
    for rgb in range(block << 16, (block + 1) << 16):
        hct = Hct.from_int(0xFF000000 | rgb)
        crc = zlib.crc32(struct.pack("<ddd", hct.hue, hct.chroma, hct.tone), crc)
    return f"{crc:08x}"


def hct_crcs():
    with Pool() as pool:
        return pool.map(hct_block_crc, range(256))


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("binary", nargs="?", help="terminal_colors_test to check")
    parser.add_argument("--script", type=Path, default=SCRIPT)
    parser.add_argument("--termscheme", type=Path, default=TERMSCHEME)
    parser.add_argument("--random", type=int, default=5000, help="number of in-process cases")
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--hct", action="store_true", help="also check Hct(argb) for every sRGB color")
    parser.add_argument("--write", type=Path, help="write cases.txt/expected.txt here instead of running")
    args = parser.parse_args()
    if not args.binary and not args.write:
        parser.error("give the test binary or --write")

    sets = [
        ("end to end (generate_colors_material.py runs)", end_to_end_cases(args.script, args.termscheme)),
        ("in process (the script's functions, random inputs)", in_process_cases(args.script, args.termscheme, args.random, args.seed)),
    ]

    if args.write:
        args.write.mkdir(parents=True, exist_ok=True)
        cases = [case for _, cases in sets for case in cases]
        (args.write / "cases.txt").write_text("".join(line + "\n" for _, line, _ in cases))
        (args.write / "expected.txt").write_text("".join(" ".join(exp) + "\n" for _, _, exp in cases))
        print(f"wrote {len(cases)} cases to {args.write}")
        if args.hct:
            (args.write / "expected-hct.txt").write_text("".join(crc + "\n" for crc in hct_crcs()))
            print(f"wrote {args.write / 'expected-hct.txt'}")
        return 0

    failed = False
    for title, cases in sets:
        out = subprocess.run(
            [args.binary], input="".join(line + "\n" for _, line, _ in cases),
            capture_output=True, text=True, check=True,
        ).stdout.splitlines()
        if len(out) != len(cases):
            sys.exit(f"{title}: expected {len(cases)} lines from the binary, got {len(out)}")
        bad_cases = bad_colors = 0
        for (label, _, expected), got_line in zip(cases, out):
            got = got_line.split()
            diffs = [(name, e, g) for name, e, g in zip(NAMES, expected, got) if e != g]
            if diffs:
                bad_cases += 1
                bad_colors += len(diffs)
                if bad_cases <= 10:
                    print(f"MISMATCH {label}: " + ", ".join(f"{n} python {e} c++ {g}" for n, e, g in diffs))
        print(f"{title}: {len(cases)} cases, {len(cases) * 16} colors, {bad_colors} differ ({bad_cases} cases)")
        failed |= bad_colors > 0

    if args.hct:
        got = subprocess.run([args.binary, "--hct-crc"], capture_output=True, text=True, check=True).stdout.split()
        bad = sum(g != e for g, e in zip(got, hct_crcs())) + abs(len(got) - 256)
        print(f"Hct(argb), all 16777216 sRGB colors bit for bit: {bad} of 256 blocks differ")
        failed |= bad > 0

    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
