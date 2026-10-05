#!/usr/bin/env bash
# Builds songrec.exe (SongRec's recognizer, GPL-3.0-or-later) for x86_64-pc-windows-msvc
# with cargo-xwin, into bin/ next to this script. songrec-win/ is the GLib-free command-line
# front end around SongRec's own fingerprinting code, which is read from upstream/ (a clone
# of https://github.com/marin-m/SongRec at $SONGREC_TAG).
set -euo pipefail

SONGREC_TAG="${SONGREC_TAG:-0.7.5}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IIW="$(cd "$HERE/../.." && pwd)"

if [ ! -d "$HERE/upstream/.git" ]; then
	git clone https://github.com/marin-m/SongRec "$HERE/upstream"
fi
git -C "$HERE/upstream" fetch --tags --quiet || true
git -C "$HERE/upstream" checkout --quiet "$SONGREC_TAG"

export XWIN_CACHE_DIR="${XWIN_CACHE_DIR:-$IIW/toolchain/xwin-cache}"
cd "$HERE/songrec-win"
cargo xwin build --release --target x86_64-pc-windows-msvc -j "${JOBS:-4}"

mkdir -p "$HERE/bin"
cp target/x86_64-pc-windows-msvc/release/songrec.exe "$HERE/bin/"
cp "$HERE/upstream/LICENSE" "$HERE/bin/COPYING"
cat > "$HERE/bin/SOURCE.txt" <<TXT
songrec.exe: SongRec's song recognizer for Windows, GPL-3.0-or-later (see COPYING).
SongRec: https://github.com/marin-m/SongRec (tag $SONGREC_TAG, $(git -C "$HERE/upstream" rev-parse HEAD))
Windows command-line front end (songrec-win): shipped in the ii-windows source tree.
TXT
echo "built $HERE/bin/songrec.exe"
