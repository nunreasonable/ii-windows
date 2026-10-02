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
- **Notification listener, unpackaged (25H2/26200).** `GetAccessStatus` is already `Allowed` when
  Settings > Privacy & security > Notifications > "Notification access" is on (the default;
  `ConsentStore\userNotificationListener` Value=Allow), and `RequestAccessAsync` returns at once,
  no prompt, no per-app entry recorded. `GetNotificationsAsync(Toast)` (~10-25 ms) and
  `RemoveNotification` work; `NotificationChanged` throws 0x80070490, so the mirror polls every 2 s.
  Only text elements are exposed (no images, no launch args, no buttons); `AppInfo.DisplayInfo.GetLogo`
  gives a PNG for packaged senders and null for unpackaged ones (PowerShell). Windows keeps showing
  its own banners; users turn them off per app (or Do not disturb) in Settings > System >
  Notifications, `Quickshell.Windows.NotificationSettings.openSettings()`.

## Status

- Phase 0 (toolchain, VM channel) and Phase 1 (core + window backend) done and verified on the
  RTX VM: AppBar reservation (also released on hide/exit), transparency, layers, input masks, IPC.
- Phase 2 done: ii boots (`vm.sh ii start`), bar + right sidebar render.
- Phase 3 merged (native backends, see the table): window tracker + virtual desktops + native
  Quickshell.Hyprland/ToplevelManager, hotkeys (RegisterHotKey + LL hook, lone Super), focus grabs,
  Core Audio, GSMTC, UPower/power modes, Start-menu apps + shell icons, system helpers
  (Quickshell.Windows: session, stats, brightness, keyboard, clipboard, credentials, input, night
  light). Still shims: SystemTray, Polkit, Pam.
- On Windows ii doesn't talk DDC/CI at startup: a boot-time query plus the monitor's OSD froze the
  ASUS VG259Q5A firmware. Brightness DDC only runs when the user changes it.
- Fixed upstream bugs that only show on named pipes: chunked IPC commands left a QDataStream
  transaction open, the client ignored responses buffered with a closed pipe, and deleting the
  server connection aborted replies in flight.

## Module mapping

| Linux (Quickshell / ii)                 | Windows                                                              | Status |
|-----------------------------------------|----------------------------------------------------------------------|--------|
| PanelWindow + WlrLayershell             | `WinPanelWindow`: AppBar, z-bands, DWM attrs, input mask via hook     | done   |
| HyprlandFocusGrab                       | outside-click / foreground-change dismissal                          | done   |
| GlobalShortcut + Hyprland keybinds      | RegisterHotKey + WH_KEYBOARD_LL (lone Super), `defaults/windows/keybinds.json` | todo |
| Hyprland workspaces / dispatch          | virtual desktops (registry + IVirtualDesktopManager + VirtualDesktopAccessor) | todo |
| HyprlandData (hyprctl -j)               | window tracker (SetWinEventHook) shaped like hyprctl JSON            | done   |
| ToplevelManager                         | same window tracker                                                  | done   |
| ScreencopyView                          | Windows.Graphics.Capture -> D3D11 texture -> QSGTexture              | built, needs GUI test |
| IdleInhibitor                           | PowerCreateRequest/PowerSetRequest                                   | done   |
| Services.Pipewire                       | Core Audio (endpoints, sessions, IPolicyConfig)                      | done   |
| Services.UPower (+PowerProfiles)        | GetSystemPowerStatus, power notifications, overlay schemes           | done   |
| Services.Mpris                          | GlobalSystemMediaTransportControls (C++/WinRT)                       | done   |
| Services.Notifications                  | native server: `notifySend` (notify-send args) + UserNotificationListener mirror of Windows toasts | built, needs GUI test |
| Services.SystemTray                     | empty stub (tray stays on the native taskbar)                        | todo   |
| Services.Polkit / Pam / WlSessionLock   | inert stubs; lock = LockWorkStation                                  | todo   |
| Quickshell.Bluetooth                    | WinRT DeviceWatcher / pairing, IKsControl reconnect                  | built, needs adapter test |
| DesktopEntries / iconPath               | FOLDERID_AppsFolder, IShellItemImageFactory provider                 | done   |
| org.kde.kirigami (Icon only)            | QML shim                                                             | done   |
| org.kde.syntaxhighlighting              | QML shim (no highlighting)                                           | done   |
| nmcli (Network.qml)                     | WlanAPI + INetworkListManager                                        | todo   |
| ddcutil/brightnessctl                   | WMI + DDC/CI (dxva2)                                                 | done   |
| cliphist/wl-copy                        | own history via AddClipboardFormatListener                           | done   |
| ydotool                                 | SendInput                                                            | done   |
| grim/slurp/magick/tesseract             | WGC capture, C++ crop, Windows.Media.Ocr                             | todo   |
| wf-recorder                             | ffmpeg ddagrab                                                       | todo   |
| matugen + switchwall.sh                 | matugen.exe + IDesktopWallpaper + system dark mode                   | todo   |
| systemctl/loginctl (Session.qml)        | LockWorkStation, ExitWindowsEx, InitiateShutdownW, SetSuspendState   | done   |
| secret-tool                             | CredRead/CredWrite                                                   | done   |
| hyprsunset                              | gamma ramp                                                           | done   |
| /proc (ResourceUsage)                   | GetSystemTimes, GlobalMemoryStatusEx, PDH                            | done   |

## ii files changed from upstream

(keep this list current — these are the merge-conflict hot spots)

- `modules/common/Platform.qml` — new file: `isWindows` singleton so Linux-only startup code can be guarded with one check instead of scattered `Qt.platform.os`
- `modules/common/Directories.qml` — Windows-safe config/state/cache/temp roots (GenericConfigLocation, TempLocation), Windows account-picture lookup, skip the Linux mkdir/rm cleanup block on Windows (FileView creates parent dirs on write)
- `modules/waffle/looks/WUserAvatar.qml` — use the Windows account-picture path instead of /var/lib/AccountsService on Windows
- `modules/common/functions/Session.qml` — Windows branches for lock/suspend/logout/poweroff/reboot/hibernate/rebootToFirmware (rundll32/shutdown)
- `modules/common/panels/lock/LockContext.qml` — skip the fprintd fingerprint-check process on Windows
- `services/Notifications.qml` — added `sendDesktop()` helper (notify-send on Linux; on Windows the native `NotificationServer.notifySend()`, which parses the same options)
- `services/MaterialThemeLoader.qml` — quiet (no printed error) fallback to Appearance's built-in palette when colors.json is missing on Windows
- `services/Hyprsunset.qml` — no-op hyprctl/hyprsunset calls on Windows (gamma ramp backend is a later phase)
- `services/FirstRunExperience.qml` — skip the switchwall.sh/qs-welcome launch on Windows
- `services/ConflictKiller.qml` — skip the kded6/mako/dunst pidof check on Windows
- `services/Cliphist.qml` — `refresh()` no-ops on Windows (no cliphist binary yet)
- `services/Updates.qml` — disable the checkupdates Timer/Process on Windows
- `services/HyprlandData.qml` — `updateAll()` no-ops on Windows, leaving window/workspace data empty
- `services/HyprlandXkb.qml` — skip hyprctl/xkb base.lst polling on Windows
- `services/HyprlandKeybinds.qml` — skip hyprctl binds polling on Windows
- `services/ResourceUsage.qml` — disable the /proc polling Timer/Process on Windows (stays at zeroed defaults)
- `services/SystemInfo.qml` — set distro/username/desktop info directly from Windows env vars instead of /etc/os-release + whoami
- `services/Network.qml` — disable all nmcli Processes on Windows (neutral/disconnected defaults), `openPublicWifiPortal()` uses `Qt.openUrlExternally` on Windows
- `services/Brightness.qml` — skip the ddcutil detect pass on Windows
- `services/SessionWarnings.qml` — `refresh()` no-ops on Windows (no pidof)
- `services/Weather.qml` — `getData()` no-ops on Windows (curl|jq pipeline needs bash+jq)
- `services/Translation.qml` — skip the `find`-based language scan on Windows, keep the en_US default
- `services/DateTime.qml` — disable the /proc/uptime polling Timer on Windows
- `services/Ai.qml` — disable the four eager `ls`/`bash` lookups (ollama models, prompts, saved chats) on Windows; this singleton is created at boot because the left sidebar's AiChat tab is always instantiated
- `services/Battery.qml` — route notify-send calls through `Notifications.sendDesktop()`, use `Session.suspend()` instead of a raw systemctl/loginctl call
- `services/TimerService.qml`, `services/LauncherSearch.qml`, `welcome.qml`, `modules/ii/bar/weather/WeatherBar.qml`, `modules/ii/sidebarLeft/aiChat/MessageCodeBlock.qml`, `modules/common/models/quickToggles/CloudflareWarpToggle.qml`, `modules/ii/sidebarRight/quickToggles/classicStyle/CloudflareWarp.qml` — route notify-send calls through `Notifications.sendDesktop()`
- `defaults/windows/colors.json` — new file: seed Material palette (copied from a live Linux colors.json; contains no personal data) so MaterialThemeLoader has something to load before matugen exists on Windows

### ii files changed for Bluetooth / capture (2026-10-02)
- Null-guarded writes to `Bluetooth.defaultAdapter` (Windows PCs often have no adapter): `modules/common/models/quickToggles/BluetoothToggle.qml`, `modules/ii/sidebarRight/quickToggles/classicStyle/BluetoothToggle.qml`, `modules/ii/sidebarRight/SidebarRightContent.qml`, `modules/waffle/actionCenter/bluetooth/BluetoothControl.qml`.
- `modules/waffle/actionCenter/nightLight/NightLightControl.qml`: dropped the Bluetooth discovery start/stop copied from the Bluetooth panel (upstream bug; it scanned while the night light panel was open).
- To do when wiring the region selector: `RegionSelection.qml` names temp files `image-${screen.name}`, and Windows screen names are `\\.\DISPLAY1`; sanitize before using them in paths.
