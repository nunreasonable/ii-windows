#!/usr/bin/env python3
import os, re, subprocess, sys
root = sys.argv[1] if len(sys.argv) > 1 else "ii"
files = subprocess.check_output(["git", "-C", root, "ls-files", "*.qml"], text=True).split()
singletons = {}
for f in files:
    src = open(os.path.join(root, f), encoding="utf-8").read()
    if re.search(r"^\s*pragma Singleton", src, re.M):
        d = os.path.dirname(f)
        singletons[os.path.splitext(os.path.basename(f))[0]] = "qs" + ("." + d.replace("/", ".") if d else "")
bad = 0
for f in files:
    if f.startswith("modules/waffle/"):
        continue
    src = open(os.path.join(root, f), encoding="utf-8").read()
    code = re.sub(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|`(?:\\.|[^`\\])*`', '""', src)
    imports = set(m.group(1) for m in re.finditer(r"^\s*import\s+(qs(?:\.[\w.]+)?)\s*$", src, re.M))
    for m in re.finditer(r'^\s*import\s+"([^"]+)"', src, re.M):
        rel = os.path.normpath(os.path.join(os.path.dirname(f), m.group(1)))
        imports.add("qs" if rel == "." else "qs." + rel.replace("/", "."))
    here = "qs" + ("." + os.path.dirname(f).replace("/", ".") if os.path.dirname(f) else "")
    for name, module in singletons.items():
        if os.path.splitext(os.path.basename(f))[0] == name:
            continue
        if not re.search(r"(?<![\w.])" + name + r"\.\w", code):
            continue
        if module in imports or module == here:
            continue
        bad += 1
        print(f"{f}: uses {name} without import {module}")
imported_dirs = {""}
for f in files:
    src = open(os.path.join(root, f), encoding="utf-8").read()
    for m in re.finditer(r"^\s*import\s+qs((?:\.[\w]+)*)", src, re.M):
        imported_dirs.add(m.group(1).lstrip(".").replace(".", "/"))
    for m in re.finditer(r'^\s*import\s+"([^"]+)"', src, re.M):
        rel = os.path.normpath(os.path.join(os.path.dirname(f), m.group(1)))
        imported_dirs.add("" if rel == "." else rel)
types_by_dir = {}
for f in files:
    name = os.path.splitext(os.path.basename(f))[0]
    if name[:1].isupper():
        types_by_dir.setdefault(os.path.dirname(f), set()).add(name)
unreachable = 0
for f in files:
    d = os.path.dirname(f)
    if d in imported_dirs or f.startswith("modules/waffle/") or "/" not in f:
        continue
    src = open(os.path.join(root, f), encoding="utf-8").read()
    me = os.path.splitext(os.path.basename(f))[0]
    used = [t for t in types_by_dir.get(d, ()) if t != me and re.search(r"(?<![\w.])" + t + r"\s*\{", src)]
    if used:
        unreachable += 1
        print(f"{f}: uses sibling types {sorted(used)} but no file imports qs.{d.replace('/', '.')} (no qmldir gets made)")
bad += unreachable
print(f"{bad} problems ({unreachable} unreachable directories)", file=sys.stderr)
sys.exit(1 if bad else 0)
