#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/env.sh"

D="$IIW/dist/ii-windows"
SHIMS="${SHIMS:-$IIW/quickshell/shims}"
[ -d "$SHIMS" ] || SHIMS="$IIW/quickshell-shims/shims"

"$IIW/tools/deploy.sh" "$D" "$IIW/build/qs/qs.exe" "$IIW/build/qs/qsw.exe" >/dev/null

mkdir -p "$D/fonts"
for f in \
	/usr/share/fonts/google-sans-flex-vf-fonts/GoogleSansFlex-VariableFont_GRAD,ROND,opsz,slnt,wdth,wght.ttf \
	/usr/share/fonts/jetbrains-mono-nerd-fonts/JetBrainsMonoNerdFont-*.ttf \
	"/usr/share/fonts/google-material-symbols-vf-rounded-fonts/MaterialSymbolsRounded[FILL,GRAD,opsz,wght].ttf" \
	"/usr/share/fonts/google-readex-pro-vf-fonts/Readexpro[HEXP,wght].ttf" \
	/usr/share/fonts/google-rubik-vf-fonts/Rubik*.ttf \
	/usr/share/fonts/florian-karsten-space-grotesk-fonts/SpaceGrotesk-*.otf; do
	cp -f "$f" "$D/fonts/"
done

rm -rf "$D/icons"
mkdir -p "$D/icons"
cp -f "$IIW"/ii/defaults/windows/icons/*.svg "$D/icons/"
cp -f "$IIW"/ii/assets/icons/*.svg "$D/icons/"
for f in "$IIW"/tools/icons/adwaita/*; do
	case "$f" in
	*.svg) cp -f "$f" "$D/icons/" ;;
	*) cp -f "$f" "$D/icons/adwaita-$(basename "$f")" ;;
	esac
done

rm -rf "$D/qml/Quickshell" "$D/qml/org" "$D/qml/_common"
if [ -d "$SHIMS" ]; then
	(cd "$SHIMS" && find . -type f ! -name '*.md' -print0) | while IFS= read -r -d '' f; do
		mkdir -p "$D/qml/$(dirname "$f")"
		cp -f "$SHIMS/$f" "$D/qml/$f"
	done
else
	echo "warning: no shims found ($SHIMS)" >&2
fi

if [ -f "$IIW/toolchain/VirtualDesktopAccessor.dll" ]; then
	cp -f "$IIW/toolchain/VirtualDesktopAccessor.dll" "$D/"
fi
if [ -f "$IIW/toolchain/win10/VirtualDesktopAccessor.dll" ]; then
	mkdir -p "$D/win10"
	cp -f "$IIW/toolchain/win10/VirtualDesktopAccessor.dll" "$D/win10/"
fi

if [ -f "$IIW/toolchain/matugen.exe" ]; then
	cp -f "$IIW/toolchain/matugen.exe" "$D/"
else
	echo "warning: no matugen.exe (toolchain/matugen.exe) - Windows wallpaper theming will fail" >&2
fi

if [ -f "$IIW/tools/songrec/bin/songrec.exe" ]; then
	cp -f "$IIW/tools/songrec/bin/songrec.exe" "$D/"
	mkdir -p "$D/licenses/songrec"
	cp -f "$IIW/tools/songrec/bin/COPYING" "$IIW/tools/songrec/bin/SOURCE.txt" "$D/licenses/songrec/"
else
	echo "warning: no tools/songrec/bin/songrec.exe - music recognition will be unavailable" >&2
fi

if [ -f "$IIW/toolchain/microtex/LaTeX.exe" ]; then
	cp -f "$IIW/toolchain/microtex/LaTeX.exe" "$D/"
	rm -rf "$D/res" && cp -r "$IIW/toolchain/microtex/res" "$D/res"
	mkdir -p "$D/licenses/microtex"
	cp -rf "$IIW"/toolchain/microtex/licenses/. "$D/licenses/microtex/"
else
	echo "warning: no toolchain/microtex/LaTeX.exe - LaTeX rendering in the AI chat will be unavailable" >&2
fi

rm -rf "$D/config/ii"
mkdir -p "$D/config"
(cd "$IIW/ii" && git ls-files -z --recurse-submodules) | while IFS= read -r -d '' f; do
	mkdir -p "$D/config/ii/$(dirname "$f")"
	cp -f "$IIW/ii/$f" "$D/config/ii/$f"
done

QMLCACHE_GEN="${QMLCACHE_GEN:-$IIW/quickshell/tools/qmlcache-bundle.py}"
if [ -f "$QMLCACHE_GEN" ]; then
	python3 "$QMLCACHE_GEN" "$D/config/ii" -q \
		--qmlcachegen "$QT_HOST/libexec/qmlcachegen" --qt-version "$QT_VERSION" ||
		echo "warning: no QML bytecode bundle - ii will compile its QML from source at startup" >&2
else
	echo "warning: no $QMLCACHE_GEN - ii will compile its QML from source at startup" >&2
fi

rm -rf "$D/testconfigs"; cp -r "$IIW/tools/testconfigs" "$D/testconfigs"
echo "staged $D ($(du -sh "$D" | cut -f1))"
