#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/env.sh"

dest="$1"; shift
mkdir -p "$dest"
for exe in "$@"; do cp -f "$exe" "$dest/"; done

unused_dll() {
	case "$(basename "$1")" in
	Qt6Designer*.dll | Qt6Help.dll | Qt6Multimedia*.dll | Qt6QmlCompiler.dll | Qt6Quick3DSpatialAudio.dll | \
		Qt6QuickTest.dll | Qt6SpatialAudio.dll | Qt6Test.dll | Qt6UiTools.dll | opengl32sw.dll | d3dcompiler_47.dll | \
		avcodec-*.dll | avformat-*.dll | avutil-*.dll | swresample-*.dll | swscale-*.dll) return 0 ;;
	esac
	return 1
}

for f in "$QT_WIN"/bin/*.dll; do
	base="${f%d.dll}"
	if [[ "$f" == *d.dll && -f "$base.dll" ]]; then continue; fi
	if unused_dll "$f"; then rm -f "$dest/$(basename "$f")"; continue; fi
	cp -f "$f" "$dest/"
done

copy_tree() {
	local src="$1" dst="$2"
	(cd "$src" && find . -type f ! -name '*.pdb' ! -name '*.lib' ! -name '*.prl' -print0) |
		while IFS= read -r -d '' rel; do
			if [[ "$rel" == *d.dll && -f "$src/${rel%d.dll}.dll" ]]; then continue; fi
			mkdir -p "$dst/$(dirname "$rel")"
			cp -f "$src/$rel" "$dst/$rel"
		done
}

rm -rf "$dest/plugins/multimedia" "$dest/qml/QtMultimedia"
rm -rf "$dest/plugins/position" "$dest/qml/Qt/test" "$dest/qml/QtQuick/Controls/Imagine" "$dest/qml/QtQuick/Controls/Universal"
for p in platforms styles imageformats iconengines tls generic networkinformation; do
	[ -d "$QT_WIN/plugins/$p" ] && copy_tree "$QT_WIN/plugins/$p" "$dest/plugins/$p"
done
rm -f "$dest"/plugins/imageformats/{qicns,qtga,qwbmp}.dll "$dest/plugins/platforms/qdirect2d.dll" \
	"$dest/plugins/tls/qopensslbackend.dll"
for q in QtQuick QtQml QtCore Qt5Compat QtPositioning Qt; do
	[ -d "$QT_WIN/qml/$q" ] && copy_tree "$QT_WIN/qml/$q" "$dest/qml/$q"
done
rm -rf "$dest/qml/Qt/test" "$dest/qml/QtQuick/Controls/Imagine" "$dest/qml/QtQuick/Controls/Universal"
if [ -d "$IIW/toolchain/vcredist" ]; then
	cp -f "$IIW"/toolchain/vcredist/*.dll "$dest/"
else
	echo "warning: no toolchain/vcredist - qs.exe won't start where the VC++ runtime isn't installed" >&2
fi

printf '[Paths]\nPrefix = .\nPlugins = plugins\nQmlImports = qml\n' > "$dest/qt.conf"
echo "deployed to $dest ($(du -sh "$dest" | cut -f1))"
