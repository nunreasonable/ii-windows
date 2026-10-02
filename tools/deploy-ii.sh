#!/usr/bin/env bash
# deploy-ii.sh: stage dist/ii-windows = qs.exe/qsw.exe + Qt runtime + bundled fonts + QML shims
# + the ii config. Push it with `tools/vm.sh push ii-windows`.
set -euo pipefail
. "$(dirname "$0")/env.sh"

D="$IIW/dist/ii-windows"
SHIMS="${SHIMS:-$IIW/quickshell/shims}"
[ -d "$SHIMS" ] || SHIMS="$IIW/quickshell-shims/shims"

"$IIW/tools/deploy.sh" "$D" "$IIW/build/qs/qs.exe" "$IIW/build/qs/qsw.exe" >/dev/null

# Fonts ii expects (all OFL/Apache, so they can ship with the build). qs loads <exe dir>/fonts.
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

# Shim modules (same URIs as the Linux-only Quickshell modules) go next to Qt's own QML modules.
# Drop the previously staged ones first: a shim deleted because a native module replaced it must
# not linger in dist with a qmldir of the same URI.
rm -rf "$D/qml/Quickshell" "$D/qml/org" "$D/qml/_common"
if [ -d "$SHIMS" ]; then
	(cd "$SHIMS" && find . -type f ! -name '*.md' -print0) | while IFS= read -r -d '' f; do
		mkdir -p "$D/qml/$(dirname "$f")"
		cp -f "$SHIMS/$f" "$D/qml/$f"
	done
else
	echo "warning: no shims found ($SHIMS)" >&2
fi

# Optional: VirtualDesktopAccessor.dll (Ciantic, MIT) next to qs.exe lets the Hyprland module
# switch virtual desktops and move other applications' windows between them without injecting
# keystrokes. Get the build for the VM's Windows version from
# https://github.com/Ciantic/VirtualDesktopAccessor/releases (2024-12-16-windows11 or newer for
# 24H2+; if every call returns -1 on a newer build, build the `rust` branch with cargo instead)
# and drop it in toolchain/.
if [ -f "$IIW/toolchain/VirtualDesktopAccessor.dll" ]; then
	cp -f "$IIW/toolchain/VirtualDesktopAccessor.dll" "$D/"
fi

# matugen.exe (GPL-2.0-or-later, InioX/matugen, cross-built for x86_64-pc-windows-msvc with
# `cargo xwin`) generates the Material You palette from the wallpaper/color, same as `matugen`
# on the Linux side. Wallpapers.qml's Windows path calls it with ii/defaults/windows/matugen.toml.
if [ -f "$IIW/toolchain/matugen.exe" ]; then
	cp -f "$IIW/toolchain/matugen.exe" "$D/"
else
	echo "warning: no matugen.exe (toolchain/matugen.exe) - Windows wallpaper theming will fail" >&2
fi

# The ii config itself (the vm job mirrors it to %LOCALAPPDATA%\quickshell\ii).
rm -rf "$D/config/ii"
mkdir -p "$D/config"
(cd "$IIW/ii" && git ls-files -z --recurse-submodules) | while IFS= read -r -d '' f; do
	mkdir -p "$D/config/ii/$(dirname "$f")"
	cp -f "$IIW/ii/$f" "$D/config/ii/$f"
done

rm -rf "$D/testconfigs"; cp -r "$IIW/tools/testconfigs" "$D/testconfigs"
echo "staged $D ($(du -sh "$D" | cut -f1))"
