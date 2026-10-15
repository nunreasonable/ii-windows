#!/usr/bin/env python3
import os, re, subprocess, sys
root = sys.argv[1] if len(sys.argv) > 1 else "ii"
base = sys.argv[2] if len(sys.argv) > 2 else None
if base:
    files = subprocess.check_output(["git", "-C", root, "diff", "--name-only", "--diff-filter=AM", base, "HEAD", "--", "*.qml"], text=True).split()
else:
    files = subprocess.check_output(["git", "-C", root, "ls-files", "*.qml"], text=True).split()
skip_top = {"shell.qml", "settings.qml", "welcome.qml", "killDialog.qml", "compilecheck.qml"}
every = subprocess.check_output(["git", "-C", root, "ls-files", "*.qml", "*.js"], text=True).split()
texts = {f: open(os.path.join(root, f), encoding="utf-8").read() for f in every if os.path.exists(os.path.join(root, f))}
def used(f):
    stem = os.path.splitext(os.path.basename(f))[0]
    pattern = re.compile(r"\b" + re.escape(stem) + r"\b")
    return any(pattern.search(t) for g, t in texts.items() if g != f)
targets, dirs = [], set()
for f in files:
    if f in skip_top or not os.path.exists(os.path.join(root, f)):
        continue
    src = open(os.path.join(root, f), encoding="utf-8").read()
    if "pragma Singleton" in src:
        continue
    if not used(f):
        print(f"skipping {f}: nothing uses it", file=sys.stderr)
        continue
    targets.append(f)
for f in subprocess.check_output(["git", "-C", root, "ls-files", "*.qml"], text=True).split():
    d = os.path.dirname(f)
    if d and not d.startswith(("defaults", "translations", "scripts")):
        dirs.add(d)
imports = "\n".join(sorted(f"import qs.{d.replace('/', '.')}" for d in dirs))
lst = ",\n        ".join(f'"{t}"' for t in sorted(targets))
print(f'''import QtQuick
import Quickshell
import qs
import qs.services
{imports}

ShellRoot {{
    Component.onCompleted: {{
        const files = [
        {lst}
        ];
        let bad = 0;
        for (const f of files) {{
            const c = Qt.createComponent(Qt.resolvedUrl(f), Component.PreferSynchronous);
            if (c.status === Component.Error) {{
                bad++;
                console.warn("COMPILE-FAIL " + f + " :: " + c.errorString().replace(/\\n/g, " | "));
            }}
        }}
        console.warn("COMPILE-DONE " + files.length + " files, " + bad + " failed");
        Qt.exit(0);
    }}
}}''')
