#!/usr/bin/env bash
# release.sh: the files of a GitHub release, in dist/release/:
#   ii-windows-<version>.zip         dist/ii-windows (staged by tools/deploy-ii.sh) + VERSION +
#                                    ii-windows-setup.exe, the package the GUI setup installs
#   ii-windows-<version>.zip.sha256  checked by the setup before it installs anything
#   ii-windows-setup.exe             the small GUI setup users download (installer/)
# <version> is the VERSION file at the top of the repo. SKIP_SETUP_BUILD=1 reuses the last build.
set -euo pipefail
. "$(dirname "$0")/env.sh"

VERSION="$(tr -d '[:space:]' < "$IIW/VERSION")"
SRC="$IIW/dist/ii-windows"
OUT="$IIW/dist/release"
SETUP="$IIW/installer/target/x86_64-pc-windows-msvc/release/ii-windows-setup.exe"
[ -d "$SRC" ] || { echo "no dist/ii-windows - run tools/deploy-ii.sh first" >&2; exit 1; }

# The setup's own version comes from VERSION at build time; Cargo's must say the same.
CARGO_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$IIW/installer/Cargo.toml" | head -n1)"
if [ "$CARGO_VERSION" != "$VERSION" ]; then
	echo "installer/Cargo.toml says $CARGO_VERSION, VERSION says $VERSION" >&2
	exit 1
fi

if [ "${SKIP_SETUP_BUILD:-}" != 1 ]; then
	(cd "$IIW/installer" && cargo xwin build --release --target x86_64-pc-windows-msvc -p ii-windows-setup)
fi
[ -f "$SETUP" ] || { echo "no $SETUP" >&2; exit 1; }

STAGE="$IIW/build/release-stage"
rm -rf "$STAGE"
mkdir -p "$STAGE" "$OUT"

# Everything qsw.exe/qs.exe need at runtime, minus the dev-only test configs. The old
# install.ps1/uninstall.ps1 (tools/package.sh) don't go in: the GUI setup replaces them.
(cd "$SRC" && find . -mindepth 1 -maxdepth 1 ! -name testconfigs ! -name install.ps1 ! -name uninstall.ps1 -print0) |
	while IFS= read -r -d '' entry; do
		cp -a "$SRC/${entry#./}" "$STAGE/"
	done
printf '%s\n' "$VERSION" > "$STAGE/VERSION"
# The package carries its own setup: an update installs the matching one into the install dir.
cp -f "$SETUP" "$STAGE/ii-windows-setup.exe"

ZIP="ii-windows-$VERSION.zip"
rm -f "$OUT/$ZIP" "$OUT/$ZIP.sha256"
(cd "$STAGE" && zip -rqX "$OUT/$ZIP" .)
(cd "$OUT" && sha256sum "$ZIP" > "$ZIP.sha256")
cp -f "$SETUP" "$OUT/ii-windows-setup.exe"

echo "release $VERSION in $OUT:"
(cd "$OUT" && ls -l "$ZIP" "$ZIP.sha256" ii-windows-setup.exe)
