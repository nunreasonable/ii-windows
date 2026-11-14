#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/env.sh"

if [ $# -lt 1 ] || ! [[ "$1" =~ ^[1-9][0-9]*$ ]]; then
	echo "usage: tools/qml-patch.sh <patch number> [ii checkout]" >&2
	exit 2
fi
N="$1"
SRC="${2:-$IIW/ii}"
VERSION="$(tr -d '[:space:]' < "$IIW/VERSION")"
NAME="ii-qml-$VERSION-p$N"
OUT="$IIW/dist/qml-patch"
STAGE="$OUT/stage"

if [ -n "$(git -C "$SRC" status --porcelain --untracked-files=no)" ]; then
	echo "$SRC has uncommitted changes" >&2
	exit 1
fi
LOCKED="$(awk '$1 == "ii" { print $3 }' "$IIW/sources.lock")"
if ! git -C "$SRC" merge-base --is-ancestor "$LOCKED" HEAD; then
	echo "$SRC does not contain the ii commit of $VERSION ($LOCKED)" >&2
	exit 1
fi

rm -rf "$STAGE"
mkdir -p "$STAGE/ii"
(cd "$SRC" && git ls-files -z --recurse-submodules) | while IFS= read -r -d '' f; do
	mkdir -p "$STAGE/ii/$(dirname "$f")"
	cp -fp "$SRC/$f" "$STAGE/ii/$f"
done

QMLCACHE_GEN="${QMLCACHE_GEN:-$IIW/quickshell/tools/qmlcache-bundle.py}"
QMLCACHEGEN="${QMLCACHEGEN:-$QT_HOST/libexec/qmlcachegen}"
"${PYTHON:-python3}" "$QMLCACHE_GEN" "$STAGE/ii" -q --qmlcachegen "$QMLCACHEGEN" --qt-version "$QT_VERSION"

rm -f "$OUT/$NAME.zip" "$OUT/$NAME.zip.sha256"
(cd "$STAGE" && zip -qrX "$OUT/$NAME.zip" ii)
(cd "$OUT" && sha256sum "$NAME.zip" > "$NAME.zip.sha256")
rm -rf "$STAGE"

echo "$OUT/$NAME.zip ($(du -h "$OUT/$NAME.zip" | cut -f1), ii $(git -C "$SRC" rev-parse --short HEAD))"
cat "$OUT/$NAME.zip.sha256"
echo "upload: gh release upload v$VERSION $OUT/$NAME.zip"
