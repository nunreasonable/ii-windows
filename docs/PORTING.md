# ii-windows: porting notes

Port of end-4's illogical-impulse (ii) to Windows 11. Two local forks:

- `quickshell/` — Quickshell @ 7511545 (branch `windows`): core made portable + `src/windows/` backend.
- `ii/` — ii history extracted from `~/hyprrr/dots-hyprland` with
  `git-filter-repo --path dots/.config/quickshell/ii/ --path .config/quickshell/ --path-rename dots/.config/quickshell/ii/: --path-rename .config/quickshell/ii/: --path-rename .config/quickshell/:`
  (branch `windows`). To pull upstream changes: refresh the dots clone, re-run the same filter on a fresh
  copy (old commit ids stay identical), fetch it as `split` and merge into `windows`.

## Build

```
. tools/env.sh
git clone --depth 1 --branch v2.5.0 https://github.com/CLIUtils/CLI11.git toolchain/cli11   # once
cmake -S quickshell -B build/qs -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo -DCMAKE_TOOLCHAIN_FILE=$IIW/tools/clang-cl-xwin.cmake \
      -DCLI11_INCLUDE_DIR=$IIW/toolchain/cli11/include
cmake --build build/qs
tools/deploy.sh dist/ii-windows build/qs/qs.exe build/qs/qsw.exe
tools/vm.sh push dist/ii-windows
```

Toolchain: clang-cl + lld-link, MSVC CRT/SDK from `xwin splat` (run `tools/fix-xwin-case.py toolchain/xwin`
after splatting: it adds the header spellings C++/WinRT needs), Qt 6.11.2 `msvc2022_64` + host `gcc_64` from
aqtinstall (git master; 3.3.0 doesn't understand the 6.11 repository layout). CMake links with lld-link
directly (`tools/clang-cl-rules.cmake`) so no rc.exe/mt.exe is needed.

Testing: the Windows VM connects to the host, never the other way around. Its agent
(`tools/vm-agent.ps1`, installed once with `iex (ssh <linux user>@192.168.122.1 iiw-vm bootstrap | Out-String)`)
starts with the Windows logon, logs in with a key pinned to `tools/vm-gateway.sh` (forced command) and
runs PowerShell jobs queued by `tools/vm.sh push|run|kill|log|ipc|shot|job`. It works in both `win11`
and `win11-gpu` (same disk); `vm.sh` never starts `win11-gpu` itself. Screenshots are taken inside
Windows, so they show the real RTX output.

## Rules learned on the target

- **C++/WinRT runs on its own MTA thread.** Qt initializes COM as STA on the GUI thread, so
  `winrt::init_apartment(multi_threaded)` there fails with RPC_E_CHANGED_MODE, and C++/WinRT forbids
  blocking `.get()` on STA threads. Every WinRT-backed service (GSMTC, Bluetooth, notifications, OCR)
  owns a worker thread that calls `init_apartment(multi_threaded)` and posts results back to Qt.
  Verified: GSMTC works unpackaged this way.
- Qt Quick picks Direct3D 11 (`GraphicsInfo.api == 4`); translucent frameless windows work as-is.

## Module mapping

| Linux (Quickshell / ii)                 | Windows                                                              | Status |
|-----------------------------------------|----------------------------------------------------------------------|--------|
| PanelWindow + WlrLayershell             | `WinPanelWindow`: AppBar, z-bands, DWM attrs, input mask via hook     | todo   |
| HyprlandFocusGrab                       | outside-click / foreground-change dismissal                          | todo   |
| GlobalShortcut + Hyprland keybinds      | RegisterHotKey + WH_KEYBOARD_LL (lone Super), `defaults/windows/keybinds.json` | todo |
| Hyprland workspaces / dispatch          | virtual desktops (registry + IVirtualDesktopManager + VirtualDesktopAccessor) | todo |
| HyprlandData (hyprctl -j)               | window tracker (SetWinEventHook) shaped like hyprctl JSON            | todo   |
| ToplevelManager                         | same window tracker                                                  | todo   |
| ScreencopyView                          | Windows.Graphics.Capture -> D3D11 texture -> QSGTexture              | todo   |
| IdleInhibitor                           | PowerCreateRequest/PowerSetRequest                                   | todo   |
| Services.Pipewire                       | Core Audio (endpoints, sessions, IPolicyConfig)                      | todo   |
| Services.UPower (+PowerProfiles)        | GetSystemPowerStatus, power notifications, overlay schemes           | todo   |
| Services.Mpris                          | GlobalSystemMediaTransportControls (C++/WinRT)                       | todo   |
| Services.Notifications                  | internal server (+ UserNotificationListener mirror later)            | todo   |
| Services.SystemTray                     | empty stub (tray stays on the native taskbar)                        | todo   |
| Services.Polkit / Pam / WlSessionLock   | inert stubs; lock = LockWorkStation                                  | todo   |
| Quickshell.Bluetooth                    | WinRT DeviceWatcher / pairing, IKsControl reconnect                  | todo   |
| DesktopEntries / iconPath               | FOLDERID_AppsFolder, IShellItemImageFactory provider                 | todo   |
| org.kde.kirigami (Icon only)            | QML shim                                                             | todo   |
| org.kde.syntaxhighlighting              | QML shim (no highlighting)                                           | todo   |
| nmcli (Network.qml)                     | WlanAPI + INetworkListManager                                        | todo   |
| ddcutil/brightnessctl                   | WMI + DDC/CI (dxva2)                                                 | todo   |
| cliphist/wl-copy                        | own history via AddClipboardFormatListener                           | todo   |
| ydotool                                 | SendInput                                                            | todo   |
| grim/slurp/magick/tesseract             | WGC capture, C++ crop, Windows.Media.Ocr                             | todo   |
| wf-recorder                             | ffmpeg ddagrab                                                       | todo   |
| matugen + switchwall.sh                 | matugen.exe + IDesktopWallpaper + system dark mode                   | todo   |
| systemctl/loginctl (Session.qml)        | LockWorkStation, ExitWindowsEx, InitiateShutdownW, SetSuspendState   | todo   |
| secret-tool                             | CredRead/CredWrite                                                   | todo   |
| hyprsunset                              | gamma ramp                                                           | todo   |
| /proc (ResourceUsage)                   | GetSystemTimes, GlobalMemoryStatusEx, PDH                            | todo   |

## ii files changed from upstream

(keep this list current — these are the merge-conflict hot spots)
