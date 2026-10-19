#!/usr/bin/env bash
set -euo pipefail

SONGREC_TAG="${SONGREC_TAG:-0.7.5}"
TARGET="${TARGET:-x86_64-pc-windows-msvc}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IIW="$(cd "$HERE/../.." && pwd)"

if [ ! -d "$HERE/upstream/.git" ]; then
	git clone https://github.com/marin-m/SongRec "$HERE/upstream"
fi
git -C "$HERE/upstream" fetch --tags --quiet || true
git -C "$HERE/upstream" checkout --quiet "$SONGREC_TAG"

cd "$HERE/songrec-win"
if [ -n "${NATIVE:-}" ]; then
	cargo build --release --target "$TARGET" -j "${JOBS:-4}"
else
	export XWIN_CACHE_DIR="${XWIN_CACHE_DIR:-$IIW/toolchain/xwin-cache}"
	cargo xwin build --release --target "$TARGET" -j "${JOBS:-4}"
fi

mkdir -p "$HERE/bin"
cp "target/$TARGET/release/songrec.exe" "$HERE/bin/"
cp "$HERE/upstream/LICENSE" "$HERE/bin/COPYING"
cat > "$HERE/bin/SOURCE.txt" <<TXT
songrec.exe ($TARGET): SongRec's song recognizer for Windows, GPL-3.0-or-later (see COPYING).
SongRec: https://github.com/marin-m/SongRec (tag $SONGREC_TAG, $(git -C "$HERE/upstream" rev-parse HEAD))
Windows command-line front end (songrec-win): shipped in the ii-windows source tree.
TXT
echo "built $HERE/bin/songrec.exe ($TARGET)"
