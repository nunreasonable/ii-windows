#!/usr/bin/env python3
import os
import re
import sys

root = sys.argv[1]
inc_re = re.compile(rb'^\s*#\s*include\s*[<"]([^>"]+)[>"]', re.M)

dirs = [os.path.join(root, d) for d in ("crt/include", "sdk/include/ucrt", "sdk/include/um",
                                         "sdk/include/shared", "sdk/include/winrt", "sdk/include/cppwinrt")]
dirs = [d for d in dirs if os.path.isdir(d)]

index = {}
for d in dirs:
    m = {}
    for dp, _, files in os.walk(d):
        for f in files:
            rel = os.path.relpath(os.path.join(dp, f), d)
            m.setdefault(rel.lower(), rel)
    index[d] = m

wanted = set()
for d in dirs:
    for dp, _, files in os.walk(d):
        for f in files:
            try:
                with open(os.path.join(dp, f), "rb") as fh:
                    data = fh.read()
            except OSError:
                continue
            for m in inc_re.finditer(data):
                wanted.add(m.group(1).decode("latin-1").replace("\\", "/"))

impl_re = re.compile(r"^winrt/impl/(.+)\.\d\.h$")
wanted |= {f"winrt/{m.group(1)}.h" for n in list(wanted) if (m := impl_re.match(n))}

added = 0
for name in wanted:
    for d in dirs:
        target = os.path.join(d, name)
        if os.path.lexists(target):
            break
        real = index[d].get(name.lower())
        if real is None:
            continue
        os.makedirs(os.path.dirname(target), exist_ok=True)
        os.symlink(os.path.relpath(os.path.join(d, real), os.path.dirname(target)), target)
        added += 1
        break
print(f"added {added} symlinks")
