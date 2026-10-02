// Read-only console probe: prints what Wallpaper::currentWallpaper() and Wallpaper::matugenPath()
// see on this machine. Calls no setter (setWallpaper/setDarkMode/setAccentColor) - never changes
// the real wallpaper or theme.
#include <cstdio>

#include <qguiapplication.h>

#include "wallpaper.hpp"

using qs::windows::sys::Wallpaper;

int main(int argc, char** argv) {
	QGuiApplication app(argc, argv); // initializes COM on this (the GUI) thread

	auto current = Wallpaper::currentWallpaper();
	printf("currentWallpaper(): %s\n", current.isEmpty() ? "(empty)" : qPrintable(current));

	printf("isDarkMode(): %s\n", Wallpaper::isDarkMode() ? "true" : "false");

	auto matugen = Wallpaper::matugenPath();
	printf("matugenPath(): %s\n", matugen.isEmpty() ? "(not found next to this exe)" : qPrintable(matugen));

	return 0;
}
