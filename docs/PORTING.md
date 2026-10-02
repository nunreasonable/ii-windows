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
| nmcli (Network.qml)                     | WlanAPI + INetworkListManager                                        | built, needs VM test (worktree `net`) |
| ddcutil/brightnessctl                   | WMI + DDC/CI (dxva2)                                                 | done   |
| cliphist/wl-copy                        | own history via AddClipboardFormatListener                           | done   |
| ydotool                                 | SendInput                                                            | done   |
| grim/slurp/magick/tesseract             | `Screenshot.captureScreen`/`.cropToFile` (C++), `Ocr` singleton on Windows.Media.Ocr (MTA worker thread) | built, needs GUI test (worktree `region`) |
| wf-recorder                             | `scripts/videos/record.ps1` (ffmpeg gdigrab + PID file); content-region detection (find-regions-venv.sh, OpenCV) stays off on Windows | built, needs GUI test (worktree `region`) |
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
- `services/Network.qml` — Windows branch binds to `WindowsNative.network` (native `Network`
  singleton: INetworkListManager + GetAdaptersAddresses/NotifyIpInterfaceChange +
  WlanAPI, worktree `net`) instead of spawning nmcli; same public API (properties/functions)
  on both platforms. `openPublicWifiPortal()` uses `Qt.openUrlExternally` on Windows. New
  `setWifiListVisible()`/`wifiNeedsLocationPermission`/`openWifiLocationSettings()` are
  Windows-only additions, wired into `WifiDialog.qml`/`WifiControl.qml`.
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

### ii files changed for the region selector (2026-10-02, worktree `region`)
- `modules/common/WindowsNative.qml`/`WindowsNativeImpl.qml` — expose the new `Quickshell.Windows` `Screenshot` and `Ocr` singletons (screenshot was already native; Ocr is new, see quickshell changes below).
- `modules/common/functions/FileUtils.qml` — new `sanitizeFilename()`, a no-op on the Linux-style names already in use (`DP-1`, `eDP-1`) but needed for Windows screen names (`\\.\DISPLAY1`), which aren't safe to drop straight into a path segment.
- `modules/common/Directories.qml` — new `recordingPidFile` path (Windows only; the ffmpeg recorder's PID, written by `scripts/videos/record.ps1` and read back by `ScreenshotAction.qml`'s recording helpers to stop the right process).
- `modules/common/utils/TempScreenshotProcess.qml` — restructured from a bare `Process` into a small wrapper: Linux keeps the same grim `Process` unchanged; Windows calls `WindowsNative.screenshot.captureScreen()` directly (no process to spawn) and defers the same `exited(exitCode, exitStatus)` signal with `Qt.callLater`. Both branches also run screen names through `FileUtils.sanitizeFilename()` now.
- `modules/common/utils/ScreenshotAction.qml` — `getCommand()` (Linux, bash/magick/wl-copy/tesseract/satty-swappy pipeline) is untouched; added `runWindows()` as its Windows counterpart, built on the native crop/clipboard/OCR calls instead of a shell pipeline, plus `startWindowsRecording()`/`stopWindowsRecording()`/`windowsRecordingStatusCommand()` for the ffmpeg-based recorder and a `Connections` block that routes `Ocr.recognized` results to the clipboard + a notification.
- `modules/ii/regionSelector/RegionSelection.qml` — screenshotPath now sanitizes `screen.name`; `enableContentRegions` forced off on Windows (no OpenCV port yet); `checkRecordingProc`'s command and `snip()`'s action dispatch branch on `Platform.isWindows` (calling the new `ScreenshotAction.runWindows()`/recording helpers instead of `getCommand()` + `execDetached`); a new `ffmpegMissing` state shows a notification and bails instead of opening the region UI when ffmpeg isn't installed.
- `scripts/videos/record.ps1` — new file: Windows counterpart to `record.sh` (ffmpeg `gdigrab` region to Matroska, remuxed to mp4 on stop because ffmpeg can only be killed from outside; sound through a DirectShow loopback device such as Stereo Mix or a virtual cable, else records silently and ii says so; PID file instead of `pgrep`/`pkill`).
- Noticed but out of scope here: `modules/ii/screenTranslator/ScreenTranslatorPanel.qml` and `modules/waffle/screenSnip/WRegionSelectionPanel.qml` build the same unsanitized `image-${screen.name}` temp path as the region selector did; worth the same `FileUtils.sanitizeFilename()` fix when those are ported.

## Packaging

`tools/package.sh` turns a built `dist/ii-windows` (staged by `tools/deploy-ii.sh`) into
`dist/ii-windows-<date>.zip`: the whole runtime (qs.exe, qsw.exe, Qt DLLs/plugins/qml, bundled
fonts, `config/ii`, `VirtualDesktopAccessor.dll`, `matugen.exe`, `qt.conf`) minus the dev-only
`testconfigs`, plus `tools/install.ps1`/`tools/uninstall.ps1` dropped at the zip root. A user
extracts the zip anywhere and runs `install.ps1` from inside it.

```
tools/deploy-ii.sh      # stage dist/ii-windows (unchanged, see Build above)
tools/package.sh        # -> dist/ii-windows-<date>.zip
```

`install.ps1` (PowerShell 5.1, per-user, no admin — everything lives under `%LOCALAPPDATA%`):

- Stops any `qs.exe`/`qsw.exe` already running from a previous install (their DLLs would
  otherwise be locked), then mirrors (`robocopy /MIR`) the package into `%LOCALAPPDATA%\ii-windows`
  — except `install.ps1`/`uninstall.ps1` themselves, which are copied separately right after so a
  reinstall never has to overwrite its own open file mid-mirror.
- Mirrors `%LOCALAPPDATA%\ii-windows\config\ii` into `%LOCALAPPDATA%\quickshell\ii` — the same
  path `tools/vm.sh`'s `ii start` job mirrors to on the test VM, and where `qsw.exe -c ii` expects
  to find it. The user's actual settings (`%LOCALAPPDATA%\illogical-impulse\config.json`, per
  `ii/modules/common/Directories.qml`'s `shellConfig`) live in a separate directory entirely and
  this mirror never reaches it; `/XF config.json` on the mirror is a defensive backstop only, in
  case that ever changes.
- Seeds `%LOCALAPPDATA%\quickshell\State\user\generated\colors.json` from
  `config/ii/defaults/windows/colors.json` only if nothing is there yet (same rule as
  `tools/vm.sh ii start`), so a reinstall doesn't clobber a palette matugen already generated.
- Creates two per-user Start Menu shortcuts: "illogical-impulse" → `qsw.exe -c ii`, "ii Settings"
  → `qsw.exe -p "<config>\settings.qml"`.
- Autostart is opt-in only, via `-Autostart` (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`,
  value name `illogical-impulse`). Default install leaves it off; running `install.ps1` again
  without `-Autostart` turns it back off, so there's one lever rather than a separate removal switch.

`uninstall.ps1` reverses all of the above (process, shortcuts, Run key, `%LOCALAPPDATA%\ii-windows`)
and keeps the user's config (`%LOCALAPPDATA%\quickshell\ii`, `%LOCALAPPDATA%\illogical-impulse`)
unless run with `-RemoveConfig`. Shared quickshell state (colors.json, logs, crash dumps under
`%LOCALAPPDATA%\quickshell\{State,run,crashes}`) isn't ii-specific and is left alone either way.

Both scripts were written against the existing `tools/vm.sh`/`deploy-ii.sh` conventions (same
paths, same "mirror, don't just copy" approach) but are **untested on the VM** (BUILD-ONLY MODE):
checked with a manual read-through plus a bracket/quote-balance script since `pwsh` isn't
installed on this host and isn't in the Fedora 44 repos. See the end of this file for VM test
steps.

## Crash handling (Windows)

`src/windows/crash/handler.cpp` (built only for `WINDOWS_BACKEND`, gated by the same
`CRASH_HANDLER` option as the Linux handler in `src/crash/`, now valid on both platforms instead
of POSIX-only) hooks `SetUnhandledExceptionFilter`, `_set_invalid_parameter_handler`,
`std::set_terminate` and `signal(SIGABRT)` into one funnel that:

1. Writes a minidump (`MiniDumpWriteDump`, dbghelp, `MiniDumpWithDataSegs | MiniDumpWithThreadInfo`)
   to `QsPaths::crashDir(instanceId)` (the same cross-platform helper `src/crash/` uses — resolves
   under `%LOCALAPPDATA%\...\cache\crashes\<instanceId>\` on Windows) as `<instanceId>-<launchTimeMs>.dmp`.
2. Copies the live detailed log (`CrashInfo::INSTANCE.logFd`, already kept pointed at the current
   log file by the platform-neutral `logging.cpp`) and a `report.txt`
   (`qs::debuginfo::combinedInfo()` plus the exception reason/code) next to it — skipped for
   `EXCEPTION_STACK_OVERFLOW` specifically, to keep that path to just the dump.
3. Relaunches `qsw.exe`/`qs.exe -c <configPath>` via `CreateProcessW`, unless this instance crashed
   within 10 seconds of its own launch (mirrors the Linux handler's crash-loop guard exactly), then
   terminates itself.

All paths (exe, config, crash dir, dump file) are resolved once at startup
(`CrashHandler::setRelaunchInfo`, called right after `InstanceInfo::CURRENT` is populated) into
fixed-size buffers, so the filter itself never allocates — the one Qt/heap-using exception is the
supporting-files step above, which only runs once the dump is already safely on disk.
`SetThreadStackGuarantee(64 KiB)` reserves stack for the filter to run in after a stack overflow,
on the thread `init()` was called from (the Qt GUI thread); worker threads (the WinRT MTA threads
used by GSMTC/Bluetooth/notifications) aren't covered.

**Design decision — in-process dump, not a watchdog process.** A second process (debugging this
one via the Win32 Debug API, or woken by a shared event to run `MiniDumpWriteDump` against our PID
from outside) would be more robust against a genuinely corrupted stack/heap, since its own stack
is guaranteed healthy — closer to what the Linux handler gets from forking a coredump child before
doing anything risky. It was not built here: it needs a handle/shared-memory handoff across a
process boundary (what happens if the watchdog itself fails to start, how the crashed process and
watchdog agree on "done", inheritable handles vs. a named mapping) that would ship with zero
verification under BUILD-ONLY MODE. The in-process filter calling `MiniDumpWriteDump(GetCurrentProcess(), ...)`
is the standard, documented approach (it's what Microsoft's own minidump samples do) and is simple
enough to read and trust without a VM. If a future pass wants the more robust version, it's a
reasonable follow-up once the VM is back and each step can actually be exercised.

For VM testing (not run here — BUILD-ONLY MODE), `QS_DEBUG_CRASH_TEST` is an undocumented env var
checked once at startup (`maybeTriggerDebugCrash`, after `setRelaunchInfo` so the real dump/relaunch
path runs) that deliberately crashes the shell via the mode it names — see "VM test steps" below.
