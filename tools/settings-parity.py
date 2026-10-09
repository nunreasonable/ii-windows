#!/usr/bin/env python3
import argparse
import pathlib
import re
import sys

MODULES = pathlib.Path(__file__).resolve().parent.parent / "ii" / "modules"
LAYOUTS = {"ii": "settings", "end4pc": "settingsPc"}
ASSIGN = re.compile(r"Config\.options\.([A-Za-z_][A-Za-z0-9_.]*?)\s*(?:=(?!=)|\+=|-=|\.(?:push|splice|pop|shift|unshift)\()")
NESTED = re.compile(r"Config\.setNestedValue\(\s*[\"'`]([^\"'`]+)[\"'`]")


def collect(directory, root):
    keys = {}
    for path in sorted(directory.rglob("*.qml")):
        text = path.read_text(encoding="utf-8")
        for pattern in (ASSIGN, NESTED):
            for match in pattern.finditer(text):
                line = text.count("\n", 0, match.start()) + 1
                keys.setdefault(match.group(1), []).append(f"{path.relative_to(root)}:{line}")
    return keys


def main():
    parser = argparse.ArgumentParser(description="Check that both settings layouts write the same Config.options keys")
    parser.add_argument("--list", action="store_true", help="print every key with where each layout writes it")
    parser.add_argument("--root", type=pathlib.Path, default=MODULES, help="the ii/modules directory to check")
    args = parser.parse_args()

    found = {}
    for name, sub in LAYOUTS.items():
        directory = args.root / sub
        if not directory.is_dir():
            print(f"missing layout {name}: {directory}", file=sys.stderr)
            return 2
        found[name] = collect(directory, args.root)

    a, b = (found[n] for n in LAYOUTS)
    only_a = sorted(set(a) - set(b))
    only_b = sorted(set(b) - set(a))

    if args.list:
        for key in sorted(set(a) | set(b)):
            print(key)
            for name in LAYOUTS:
                for where in found[name].get(key, []):
                    print(f"    {name:7} {where}")

    names = list(LAYOUTS)
    for key in only_a:
        print(f"only in {names[0]}: {key}  ({', '.join(a[key])})")
    for key in only_b:
        print(f"only in {names[1]}: {key}  ({', '.join(b[key])})")

    common = len(set(a) & set(b))
    print(f"{common} keys in both, {len(only_a)} only in {names[0]}, {len(only_b)} only in {names[1]}")
    return 0 if not only_a and not only_b else 1


if __name__ == "__main__":
    sys.exit(main())
