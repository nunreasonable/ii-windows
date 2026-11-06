# Hacking on Illogical Windows

Thanks for wanting to help. This page says where the code is, how to build and test it, and what a pull request needs. For bugs and ideas without code, open an [issue](https://github.com/nunreasonable/ii-windows/issues) instead. Before a big change, open an issue first so we can agree on the approach.

## Where the code is

Illogical Windows is three repositories:

| What | Repository and branch | Send PRs here for |
|---|---|---|
| The setup, packaging, build scripts and CI | [nunreasonable/ii-windows](https://github.com/nunreasonable/ii-windows), `main` | `installer/` (Rust + Tauri 2), `tools/`, `.github/workflows/`, the docs |
| Quickshell for Windows | [nunreasonable/quickshell](https://github.com/nunreasonable/quickshell), `windows` | the C++ engine; the Windows backend is in `src/windows/`, and `shims/` has QML stand-ins for Linux-only modules |
| ii for Windows (the shell itself, QML) | [nunreasonable/dots-hyprland](https://github.com/nunreasonable/dots-hyprland), `ii-windows` | everything you see: bar, sidebars, widgets, settings, services |

`sources.lock` in this repo pins the commits of the two forks that a release is built from. If a change needs more than one repository, open a PR in each and link them to each other.

Changes that aren't about Windows belong upstream: [end-4/dots-hyprland](https://github.com/end-4/dots-hyprland) for ii and [Quickshell](https://github.com/quickshell-mirror/quickshell) for the engine.

## Changing only QML

You don't need to build anything to change ii's QML:

1. Install Illogical Windows with the setup.
2. Edit the files in `%LOCALAPPDATA%\quickshell\ii`. That's the copy the shell loads.
3. Delete `%LOCALAPPDATA%\quickshell\ii\.qmlcache`, so the edited files are read instead of the compiled cache.
4. Stop `qsw.exe` (Task Manager) and start *illogical-impulse* from the Start menu.
5. Copy your changes into a clone of the ii fork and open the PR from there. The setup's *Update* and *Repair* replace that folder, so don't keep your only copy there.

The logs are in `%LOCALAPPDATA%\quickshell\run\by-id\<run>\log.log`, and *Settings → About → Report a problem* zips them.

## Building

The release build runs on GitHub Actions, and [`.github/workflows/build.yml`](.github/workflows/build.yml) is the reference for every step and version.

**The easiest way to get a full package:** fork this repository and the fork you changed, point the matching line of `sources.lock` at your repository and commit, and run the *Build* workflow in your fork (*Actions → Build → Run workflow*). The artifacts have the zip and `ii-windows-setup.exe`, and the setup can install that zip with `ii-windows-setup.exe --package <zip>`.

**Building Quickshell on Windows**, in Git Bash started from an *x64 Native Tools* prompt of Visual Studio 2022 (with its C++ and Clang tools), with Python, CMake and Ninja:

```bash
while read -r name url sha; do
  git clone -q "$url" "$name" && git -C "$name" checkout -q "$sha" && git -C "$name" submodule update -q --init --recursive
done < sources.lock
pip install aqtinstall
aqt install-qt windows desktop 6.11.2 win64_msvc2022_64 -m qt5compat qtimageformats qtpositioning qtshadertools -O C:/Qt
git clone -q --depth 1 --branch v2.5.0 https://github.com/CLIUtils/CLI11.git C:/CLI11
cmake -S quickshell -B build/qs -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo \
  -DCMAKE_C_COMPILER=clang-cl -DCMAKE_CXX_COMPILER=clang-cl \
  -DCMAKE_PREFIX_PATH=C:/Qt/6.11.2/msvc2022_64 -DCLI11_INCLUDE_DIR=C:/CLI11/include
cmake --build build/qs
```

That gives `qs.exe` (the console version, for commands like `qs ipc call`) and `qsw.exe` (the one that runs the shell, without a console window). To try them, stop ii and copy them over the ones in `%LOCALAPPDATA%\ii-windows`; *Repair* in the setup puts the released ones back.

**The setup** builds with Rust: `cd installer && cargo build --release --target x86_64-pc-windows-msvc -p ii-windows-setup`.

**Cross-compiling from Linux** (clang-cl, lld-link and the MSVC SDK from xwin) is what the maintainer uses; see `tools/env.sh`, `tools/clang-cl-xwin.cmake` and `docs/AGENTS.md`.

## Testing

- Test on Windows 11. If your change touches Windows APIs (windows, the taskbar, virtual desktops, audio, notifications, blur), also test on Windows 10 22H2, which is still supported, or say in the PR that you couldn't.
- Try both visual styles if you changed something they share (*Settings → Quick → Style*), and both settings layouts if you changed a settings page.
- Put back anything your tests changed in Windows (wallpaper, theme, taskbar, exclusions).
- In the PR, say what you tested, on which Windows build, and how.

## Code

- **Match the code around yours**: formatting (`.clang-format` in Quickshell, `.qmlformat.ini` in ii), naming and patterns.
- **In ii, keep each service's public properties and functions the same** as upstream and change only how they work on Windows. Windows-only behavior goes behind `Platform.isWindows`, and Windows-only C++ goes in Quickshell's `src/windows/`.
- **C++/WinRT calls run on their own MTA worker thread**, with the results posted back to the Qt thread. Qt's GUI thread is STA, and blocking `.get()` calls aren't allowed there.
- **The Windows code has almost no comments.** Explain the why in the PR description and in the commit message when it isn't obvious.
- **Text that users see goes through `Translation.tr()`.** If you can, add the Brazilian Portuguese string to `translations/pt_BR.json`.
- **Don't copy code from projects whose license doesn't allow it here** (anything not compatible with GPL-3.0, or LGPL-3.0 for Quickshell, or with no license). MIT and Apache-2.0 code is fine with its notice and an entry in [THIRD-PARTY.md](THIRD-PARTY.md).
- **One topic per PR, with commits that say what changed in one short sentence**, for example "Keep the dock above fullscreen video on Windows 10", without prefixes like `fix:`.

## License

By opening a PR, you agree that your contribution is under the license of the repository it goes to: GPL-3.0 for this repository and for ii, LGPL-3.0 for Quickshell.

If you'd rather publish your own variant than send a PR, see [FORKING.md](FORKING.md).
