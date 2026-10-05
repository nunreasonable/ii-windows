#!/usr/bin/env bash
# Builds LaTeX.exe (MicroTeX, MIT) for ii's AI chat LaTeX rendering on Windows into
# toolchain/microtex/ (LaTeX.exe, res/, licenses/), which tools/deploy-ii.sh picks up.
#
# Upstream's `-headless` CLI only exists for the gtkmm/cairomm backend, which doesn't
# cross-compile for MSVC; microtex-windows.patch adds an equivalent Qt + QSvgGenerator CLI
# (qt_headless_main.cpp, next to this script) with the same flags and SVG output.
set -euo pipefail

MICROTEX_COMMIT="${MICROTEX_COMMIT:-0e3707f}"
TINYXML2_COMMIT="${TINYXML2_COMMIT:-8224e42}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IIW="$(cd "$HERE/../.." && pwd)"
. "$IIW/tools/env.sh"

SRC="$IIW/build/microtex-src"
TX_SRC="$IIW/build/tinyxml2-src"
TX_PREFIX="$IIW/toolchain/tinyxml2"
OUT="$IIW/toolchain/microtex"
CROSS=(-G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_TOOLCHAIN_FILE="$IIW/tools/clang-cl-xwin.cmake")

# tinyxml2 (zlib license), MicroTeX's XML parser
[ -d "$TX_SRC/.git" ] || git clone https://github.com/leethomason/tinyxml2 "$TX_SRC"
git -C "$TX_SRC" checkout --quiet "$TINYXML2_COMMIT"
cmake -S "$TX_SRC" -B "$IIW/build/tinyxml2-build" "${CROSS[@]}" \
	-Dtinyxml2_BUILD_TESTING=OFF -DCMAKE_INSTALL_PREFIX="$TX_PREFIX"
cmake --build "$IIW/build/tinyxml2-build" -j "${JOBS:-8}"
cmake --install "$IIW/build/tinyxml2-build"

# MicroTeX with the Windows headless CLI
[ -d "$SRC/.git" ] || git clone https://github.com/NanoMichael/MicroTeX "$SRC"
git -C "$SRC" checkout --quiet -- .
git -C "$SRC" checkout --quiet "$MICROTEX_COMMIT"
git -C "$SRC" apply "$HERE/microtex-windows.patch"
cp -f "$HERE/qt_headless_main.cpp" "$SRC/src/samples/"
cmake -S "$SRC" -B "$IIW/build/microtex" "${CROSS[@]}" -DQT=ON -DBUILD_EXAMPLE=OFF \
	-DCMAKE_PREFIX_PATH="$TX_PREFIX" -Dtinyxml2_DIR="$TX_PREFIX/lib/cmake/tinyxml2"
cmake --build "$IIW/build/microtex" -j "${JOBS:-8}"

mkdir -p "$OUT/licenses"
cp -f "$IIW/build/microtex/LaTeX.exe" "$OUT/"
rm -rf "$OUT/res" && cp -r "$SRC/res" "$OUT/res"
cp -f "$SRC/LICENSE" "$OUT/licenses/MicroTeX-LICENSE-MIT.txt"
cp -f "$TX_SRC/LICENSE.txt" "$OUT/licenses/tinyxml2-LICENSE.txt"
echo "built $OUT/LaTeX.exe"
