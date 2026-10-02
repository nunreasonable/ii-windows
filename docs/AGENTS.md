# Working on ii-windows (read this first)

Port of end-4's illogical-impulse (ii) Quickshell shell to Windows 11. Two local git repos:
- `quickshell/` — Quickshell fork (C++20/Qt 6.11.2), branch `windows`. Windows code lives in
  `src/windows/` (WindowsPlugin, WinPanelWindow, input masks, Quickshell.Wayland stand-ins incl. C++
  IdleInhibitor). Linux-only modules are compiled out; `shims/` holds pure-QML stand-ins with the same
  URIs (Quickshell.Hyprland, Quickshell.Services.*, Quickshell.Bluetooth, org.kde.*), deployed to
  `<dist>/qml`. A native C++ module with the same URI replaces its shim: delete the shim dir in the same
  change.
- `ii/` — the ii QML config fork, branch `windows`. `modules/common/Platform.qml` has `isWindows`.
  Keep each service's public API identical; change implementations only.

## Rules
- Work in your OWN git worktree + branch (others work in parallel):
  `git -C $IIW/quickshell worktree add $IIW/wt-<name> -b <name> windows`
  (and the same for `ii/` if you touch QML: `wt-ii-<name>`). Never edit the main checkouts.
- Build your worktree in its own dir:
  ```
  cd $IIW && . tools/env.sh
  cmake -S wt-<name> -B build/wt-<name> -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo \
    -DCMAKE_TOOLCHAIN_FILE=$IIW/tools/clang-cl-xwin.cmake -DCLI11_INCLUDE_DIR=$IIW/toolchain/cli11/include
  cmake --build build/wt-<name>
  ```
  Toolchain: clang-cl + lld-link + xwin (MSVC CRT/SDK incl. C++/WinRT headers) + Qt 6.11.2 msvc2022_64.
- Commit often (every compiling milestone), short plain messages, NEVER a Co-Authored-By trailer, never
  push or add remotes. Sessions can end abruptly; uncommitted work is lost.
- Code style: match the surrounding code (tabs, `this->`, Q_OBJECT_BINDABLE_PROPERTY patterns, comments
  explain why). Don't copy code from GPL/AGPL projects (Seelen, Lively, quickshell-macos); Apache/MIT
  references (PowerToys, ManagedShell, EarTrumpet, Twinkle Tray, VirtualDesktopAccessor) are fine to
  consult.
- **C++/WinRT must run on its own MTA thread** (`winrt::init_apartment(multi_threaded)` on a worker
  thread; post results back to the Qt thread). Qt's GUI thread is STA; init_apartment there fails and
  blocking `.get()` is forbidden on STA. Verified on the target.
- **BUILD-ONLY MODE (since 2026-10-02):** the user needs the RTX on Linux, so both VMs (`win11`,
  `win11-gpu`) are off and must stay off. Do not start a VM and do not queue `tools/vm.sh` jobs (the
  agent is offline; jobs only time out). Verify by building (Windows cross-build, and the Linux build if
  you touch shared code), by reading the APIs' docs, and with small host-side checks where possible.
  End your report with the exact VM test steps the integrator should run later.
- (When the VM is back) Testing on the real target: a Windows 11 VM (25H2, build 26200, RTX 5060 Ti passthrough, 1920x1080)
  is reachable through `tools/vm.sh job [timeout] < script.ps1` (runs a PowerShell script in the
  logged-on user's session and prints its output). You may push a small console test program
  (build it in your build dir, stage it under `dist/<name>-probe/` with `tools/deploy.sh dist/<name>-probe
  <exe>`, `tools/vm.sh push <name>-probe`, then run it from a job) to probe Windows APIs. Do NOT start
  GUI programs or the shell (`vm.sh ii start`, `vm.sh run qsw ...`) — the integrator does GUI tests, and
  the VM desktop is shared with the user.
- **Never kill processes broadly in VM jobs** (e.g. `Get-Process powershell | Stop-Process`): the VM
  agent itself is a hidden powershell.exe running `iiw-agent\boot.ps1`; killing it cuts everyone off
  until the user logs off/on in Windows. Stop only processes you started (keep their PIDs).
- **Don't talk DDC/CI to the monitor** from VM probes (brightness/input VCP codes): it froze the
  monitor's firmware once. **Don't inject keyboard/mouse input** (SendInput) in VM probes unless the
  task is about input, and then keep it to a few seconds: the user may be using that desktop.
- The shell's state: `ii` QML boots on Windows (`tools/vm.sh ii start`); merged native backends:
  window tracker + virtual desktops + Quickshell.Hyprland/ToplevelManager, Hotkeys/GlobalShortcut,
  FocusGrab, Pipewire (Core Audio), Mpris (GSMTC), UPower, DesktopEntries/icons, Quickshell.Windows
  system helpers, Quickshell.Bluetooth (WinRT), Services.Notifications (own server + toast mirror),
  ScreencopyView on Windows.Graphics.Capture and `Quickshell.Windows.Screenshot.captureScreen()`
  (src/windows/capture.*). Network (WlanAPI) and theming (matugen/wallpaper) are in progress on
  branches `net` and `theme`. ii reaches Windows-only singletons through `modules/common/WindowsNative.qml`
  (lazy, so the files still load on Linux) — follow that pattern for new ii wiring.
- Spawning on Windows: QProcess escapes `"` inside an argument as `\"` (C runtime rules), which
  cmd.exe does not understand. Never pass `cmd /c` an argument containing quotes: call the program
  directly with separate arguments (curl.exe, ffmpeg...), or use `powershell -Command` (it parses
  `\"` correctly). Paths given to cmd built-ins need backslashes (`/x` is a switch).
- Report at the end: branch + commits, files, API mapping, what's verified on the VM, TODOs/risks.
