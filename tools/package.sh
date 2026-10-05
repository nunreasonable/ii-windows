#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/env.sh"

SRC="$IIW/dist/ii-windows"
[ -d "$SRC" ] || { echo "no dist/ii-windows - run tools/deploy-ii.sh first" >&2; exit 1; }

DATE="$(date +%Y%m%d)"
STAGE="$IIW/build/package-stage"
OUT="$IIW/dist/ii-windows-$DATE.zip"

rm -rf "$STAGE"
mkdir -p "$STAGE"

(cd "$SRC" && find . -mindepth 1 -maxdepth 1 ! -name testconfigs -print0) |
	while IFS= read -r -d '' entry; do
		cp -a "$SRC/${entry#./}" "$STAGE/"
	done

cp -f "$IIW/tools/install.ps1" "$IIW/tools/uninstall.ps1" "$STAGE/"

rm -f "$OUT"
(cd "$STAGE" && zip -rq "$OUT" .)

echo "packaged $OUT ($(du -sh "$OUT" | cut -f1))"
