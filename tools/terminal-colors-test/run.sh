#!/usr/bin/env bash
# Host check of the Windows backend's terminal color harmonization against ii's Python
# generator: builds terminal_colors_test with the host compiler (CXX=clang++ to use clang) from
# the given quickshell src/windows and runs compare.py with ii's Python.
#   tools/terminal-colors-test/run.sh [quickshell src/windows] [compare.py args...]
set -euo pipefail
IIW="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
src="${1:-$IIW/quickshell/src/windows}"
shift || true
build="$IIW/build/terminal-colors-test${CXX:+-$(basename "$CXX")}"
python="${II_PYTHON:-$HOME/.local/state/quickshell/.venv/bin/python}"

cmake -S "$IIW/tools/terminal-colors-test" -B "$build" -G Ninja -DQS_WINDOWS_SRC="$src" >/dev/null
cmake --build "$build" >/dev/null
"$python" "$IIW/tools/terminal-colors-test/compare.py" "$build/terminal_colors_test" "$@"
