import os
import sys
import zipfile

root, out = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as z:
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames.sort()
        rel_dir = os.path.relpath(dirpath, root)
        if rel_dir != ".":
            z.write(dirpath, rel_dir.replace(os.sep, "/") + "/")
        for name in sorted(filenames):
            path = os.path.join(dirpath, name)
            z.write(path, os.path.relpath(path, root).replace(os.sep, "/"))
