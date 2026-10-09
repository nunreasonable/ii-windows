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
print(f"{bad} missing singleton imports", file=sys.stderr)
sys.exit(1 if bad else 0)
