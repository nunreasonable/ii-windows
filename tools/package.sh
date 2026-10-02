#!/usr/bin/env bash
# package.sh: zip dist/ii-windows (the qs.exe/qsw.exe + Qt + fonts + qml + config/ii staged by
# tools/deploy-ii.sh) together with install.ps1/uninstall.ps1 into dist/ii-windows-<date>.zip -
# the file a user downloads and extracts, then runs install.ps1 from.
set -euo pipefail
. "$(dirname "$0")/env.sh"

SRC="$IIW/dist/ii-windows"
[ -d "$SRC" ] || { echo "no dist/ii-windows - run tools/deploy-ii.sh first" >&2; exit 1; }

DATE="$(date +%Y%m%d)"
STAGE="$IIW/build/package-stage"
OUT="$IIW/dist/ii-windows-$DATE.zip"

rm -rf "$STAGE"
mkdir -p "$STAGE"

# Everything qsw.exe/qs.exe need at runtime (qs.exe, qsw.exe, Qt DLLs/plugins/qml, fonts,
# config/ii, VirtualDesktopAccessor.dll, matugen.exe, qt.conf), minus the dev-only test configs
# that tools/vm.sh uses for manual checks - not useful to an end user and not referenced by
# install.ps1.
(cd "$SRC" && find . -mindepth 1 -maxdepth 1 ! -name testconfigs -print0) |
	while IFS= read -r -d '' entry; do
		cp -a "$SRC/${entry#./}" "$STAGE/"
	done

cp -f "$IIW/tools/install.ps1" "$IIW/tools/uninstall.ps1" "$STAGE/"

rm -f "$OUT"
(cd "$STAGE" && zip -rq "$OUT" .)

echo "packaged $OUT ($(du -sh "$OUT" | cut -f1))"
