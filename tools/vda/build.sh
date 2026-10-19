#!/usr/bin/env bash
set -euo pipefail

VDA_COMMIT="${VDA_COMMIT:-ceb648849dcc5760a7f4ed223308ffe3de0b51c6}"
TARGET="${TARGET:-x86_64-pc-windows-msvc}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IIW="$(cd "$HERE/../.." && pwd)"

SRC="$IIW/build/vda-src"
if [ ! -d "$SRC/.git" ]; then
	git init -q "$SRC"
fi
git -C "$SRC" fetch -q --depth 1 https://github.com/Ciantic/VirtualDesktopAccessor "$VDA_COMMIT"
git -C "$SRC" checkout -q FETCH_HEAD

cd "$SRC"
if [ -n "${NATIVE:-}" ]; then
	cargo build --release -p dll --target "$TARGET" -j "${JOBS:-4}"
else
	export XWIN_CACHE_DIR="${XWIN_CACHE_DIR:-$IIW/toolchain/xwin-cache}"
	cargo xwin build --release -p dll --target "$TARGET" -j "${JOBS:-4}"
fi

mkdir -p "$HERE/bin"
cp "target/$TARGET/release/VirtualDesktopAccessor.dll" "$HERE/bin/"
cp "$SRC/LICENSE.txt" "$HERE/bin/LICENSE.txt"
cat > "$HERE/bin/SOURCE.txt" <<TXT
VirtualDesktopAccessor.dll ($TARGET): built from source, MIT (see LICENSE.txt).
VirtualDesktopAccessor: https://github.com/Ciantic/VirtualDesktopAccessor (commit $VDA_COMMIT)
The GitHub release for this project only ships an x64 build, so ii-windows builds
the dll crate from source for architectures without a prebuilt release asset.
TXT
echo "built $HERE/bin/VirtualDesktopAccessor.dll ($TARGET)"
