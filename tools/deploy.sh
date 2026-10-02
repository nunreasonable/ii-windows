#!/usr/bin/env bash
# deploy.sh <dest dir> <exe...>: stage Windows executables with the Qt runtime they need,
# since windeployqt can't run on Linux. Copies release DLLs, plugins and QML modules.
set -euo pipefail
. "$(dirname "$0")/env.sh"

dest="$1"; shift
mkdir -p "$dest"
for exe in "$@"; do cp -f "$exe" "$dest/"; done

# Release DLLs only: skip Foo d.dll when Foo.dll exists.
for f in "$QT_WIN"/bin/*.dll; do
	base="${f%d.dll}"
	if [[ "$f" == *d.dll && -f "$base.dll" ]]; then continue; fi
	cp -f "$f" "$dest/"
done

copy_tree() { # copy_tree <src> <dst>, release binaries only, no pdb/static libs
	local src="$1" dst="$2"
	(cd "$src" && find . -type f ! -name '*.pdb' ! -name '*.lib' ! -name '*.prl' -print0) |
		while IFS= read -r -d '' rel; do
			if [[ "$rel" == *d.dll && -f "$src/${rel%d.dll}.dll" ]]; then continue; fi
			mkdir -p "$dst/$(dirname "$rel")"
			cp -f "$src/$rel" "$dst/$rel"
		done
}

for p in platforms styles imageformats iconengines tls generic multimedia position networkinformation; do
	[ -d "$QT_WIN/plugins/$p" ] && copy_tree "$QT_WIN/plugins/$p" "$dest/plugins/$p"
done
for q in QtQuick QtQml QtCore Qt5Compat QtPositioning QtMultimedia Qt; do
	[ -d "$QT_WIN/qml/$q" ] && copy_tree "$QT_WIN/qml/$q" "$dest/qml/$q"
done
# The MSVC C++ runtime the binaries link against, app-local: a fresh Windows 10 (or any PC
# without the Visual C++ redistributable) has none, and qs.exe wouldn't start. Taken from
# Microsoft's VC_redist.x64.exe (VS 2026, 14.51), which must be at least the version of the
# headers the build used (toolchain/xwin/crt/include/crtversion.h).
if [ -d "$IIW/toolchain/vcredist" ]; then
	cp -f "$IIW"/toolchain/vcredist/*.dll "$dest/"
else
	echo "warning: no toolchain/vcredist - qs.exe won't start where the VC++ runtime isn't installed" >&2
fi

printf '[Paths]\nPrefix = .\nPlugins = plugins\nQmlImports = qml\n' > "$dest/qt.conf"
echo "deployed to $dest ($(du -sh "$dest" | cut -f1))"
