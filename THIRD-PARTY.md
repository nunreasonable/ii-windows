# Third-party components

Everything in the ii-windows package is open source, except the Microsoft Visual C++ runtime, which is a system library. The release build fetches each component from the source below at a fixed version: see `sources.lock`, `tools/deps.lock` and `.github/workflows/build.yml`. The license texts are in the package's `licenses` folder.

| Component | Version | License | Source | In the package |
|---|---|---|---|---|
| ii-windows (setup, scripts, packaging) | this release | GPL-3.0 | https://github.com/nunreasonable/ii-windows | `ii-windows-setup.exe`, `config/ii/scripts` |
| illogical-impulse (ii), Windows port | `sources.lock` | GPL-3.0 | https://github.com/nunreasonable/dots-hyprland (branch `ii-windows`), from https://github.com/end-4/dots-hyprland | `config/ii` |
| rounded-polygon-qmljs | pinned by ii (`git submodule`) | Apache-2.0 | https://github.com/end-4/rounded-polygon-qmljs | `config/ii/modules/common/widgets/shapes` |
| Quickshell, Windows port | `sources.lock` | LGPL-3.0 | https://github.com/nunreasonable/quickshell (branch `windows`), from https://github.com/quickshell-mirror/quickshell | `qs.exe`, `qsw.exe`, `qml/Quickshell` |
| Qt | 6.11.2 | LGPL-3.0 | https://download.qt.io/archive/qt/ | `Qt6*.dll`, `plugins`, `qml` |
| cpptrace | 1.0.4 | MIT | https://github.com/jeremy-rifkin/cpptrace | built into `qs.exe`, `qsw.exe` |
| CLI11 | 2.5.0 | BSD-3-Clause | https://github.com/CLIUtils/CLI11 | built into `qs.exe`, `qsw.exe` |
| matugen | 4.1.0 | GPL-2.0 | https://github.com/InioX/matugen | `matugen.exe` |
| MicroTeX | `0e3707f` | MIT; resources GPL-3.0 and their own font licenses | https://github.com/NanoMichael/MicroTeX | `LaTeX.exe`, `res` |
| tinyxml2 | `8224e42` | Zlib | https://github.com/leethomason/tinyxml2 | built into `LaTeX.exe` |
| SongRec | 0.7.5 | GPL-3.0-or-later | https://github.com/marin-m/SongRec | `songrec.exe` |
| VirtualDesktopAccessor | `2024-12-16-windows11`, `2019-windows10` | MIT | https://github.com/Ciantic/VirtualDesktopAccessor | `VirtualDesktopAccessor.dll`, `win10` |
| Google Sans Flex | `251aa5a` | OFL-1.1 | https://github.com/end-4/google-sans-flex | `fonts` |
| Material Symbols Rounded | `bb04090` | Apache-2.0 | https://github.com/google/material-design-icons | `fonts` |
| Readex Pro | `563dfbb` | OFL-1.1 | https://github.com/ThomasJockin/readexpro | `fonts` |
| Space Grotesk | `03507d0` | OFL-1.1 | https://github.com/floriankarsten/space-grotesk | `fonts` |
| JetBrains Mono Nerd Font | 3.4.0 | OFL-1.1 (font), MIT (Nerd Fonts) | https://github.com/ryanoasis/nerd-fonts | `fonts` |
| Rubik (setup window only) | — | OFL-1.1 | https://github.com/googlefonts/rubik | inside `ii-windows-setup.exe` (`installer/ui/fonts`) |
| Adwaita icons (a few) | — | CC-BY-SA-3.0 / LGPL-3.0 | https://gitlab.gnome.org/GNOME/adwaita-icon-theme | `icons/adwaita-*` |
| Microsoft Visual C++ runtime | 14.x | Microsoft Visual C++ Redistributable (system library) | Visual Studio | `msvcp140*.dll`, `vcruntime140*.dll` |

The setup (`ii-windows-setup.exe`) is built with Tauri 2 and Rust crates under permissive licenses (MIT, Apache-2.0 and similar); `installer/Cargo.lock` lists them. It uses the Microsoft Edge WebView2 Runtime that comes with Windows. `songrec.exe` and `matugen.exe` are also Rust programs, and their crates are listed in their own `Cargo.lock`.

The ARM64 package (`ii-windows-<version>-arm64.zip`) is built from the same sources at the same versions, natively on GitHub's Windows 11 ARM64 runner: Qt's own ARM64 build (`win64_msvc2022_arm64`), Quickshell, matugen, SongRec and MicroTeX compiled for ARM64, and the ARM64 Visual C++ runtime from the runner's Visual Studio (without `vcruntime140_1.dll`, which only x64 code needs). VirtualDesktopAccessor's GitHub release only ships an x64 build, so for ARM64 `VirtualDesktopAccessor.dll` is built from the `dll` crate in its source tree at commit `ceb648849dcc5760a7f4ed223308ffe3de0b51c6` (`tools/vda/build.sh`; the same commit `tools/deps.lock` takes `licenses/VirtualDesktopAccessor/LICENSE.txt` from). The ARM64 package has no `win10` folder, and its `ii-windows-setup.exe` is the same x64 setup as in the x64 package.

The NabyScape visual style recreates the look and behavior of [NabyScape](https://github.com/BillyNabil/nabyscape) by BillyNabil (MIT) with ii's own QML and Material Symbols icons. No code or images from NabyScape are in the package.
