#!/usr/bin/env bash
set -euo pipefail
IIW="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEPS="${DEPS_DIR:-$IIW/build/deps}"
mkdir -p "$DEPS"

while read -r dest sum url; do
	[ -z "$dest" ] && continue
	out="$DEPS/$dest"
	if [ -f "$out" ] && echo "$sum  $out" | sha256sum -c --status; then continue; fi
	mkdir -p "$(dirname "$out")"
	curl -fsSL --retry 3 -o "$out.part" "$url"
	if ! echo "$sum  $out.part" | sha256sum -c --status; then
		echo "checksum mismatch: $url" >&2
		rm -f "$out.part"
		exit 1
	fi
	mv "$out.part" "$out"
done < "$IIW/tools/deps.lock"

mkdir -p "$DEPS/fonts" "$DEPS/licenses/fonts"
"${PYTHON:-python3}" - "$DEPS/archives/JetBrainsMono-nerd-3.4.0.zip" "$DEPS/fonts" "$DEPS/licenses/fonts/JetBrainsMono-OFL.txt" <<'PY'
import os, sys, zipfile
with zipfile.ZipFile(sys.argv[1]) as z:
    for name in z.namelist():
        base = os.path.basename(name)
        if base.startswith("JetBrainsMonoNerdFont-") and base.endswith(".ttf"):
            target = os.path.join(sys.argv[2], base)
        elif base == "OFL.txt":
            target = sys.argv[3]
        else:
            continue
        with z.open(name) as src, open(target, "wb") as dst:
            dst.write(src.read())
PY
echo "deps ready in $DEPS"
