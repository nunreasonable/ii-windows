#!/usr/bin/env bash
set -euo pipefail
IIW="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
src="${1:-$IIW/quickshell/src/windows}"
build="$IIW/build/recorder-audio-test${CXX:+-$(basename "$CXX")}"

cmake -S "$IIW/tools/recorder-audio-test" -B "$build" -G Ninja -DQS_WINDOWS_SRC="$src" >/dev/null
cmake --build "$build" >/dev/null
"$build/recorder_audio_test"
