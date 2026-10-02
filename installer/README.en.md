# Before you install

**illogical-impulse** (ii) is end-4's desktop shell for Hyprland on Linux: a bar, sidebars, an
overview, a launcher, notifications and Material You colors taken from your wallpaper.
**ii-windows** is an unofficial port of it to Windows 10 and 11, running on a Windows build of
Quickshell. It is experimental: expect rough edges.

This page lists what this setup changes on your computer and what ii changes while it runs.
Everything is installed for your Windows user only; no administrator permission is needed,
except for the optional PowerShell 7.

## What gets installed, and where

- **The program** goes to `%LOCALAPPDATA%\ii-windows` (about 240 MB): Quickshell (`qs.exe`,
  `qsw.exe`), the Qt libraries, the fonts and icons ii uses, `matugen.exe` (makes the color
  palette from your wallpaper), `VirtualDesktopAccessor.dll` (two builds: one for Windows 11,
  one for Windows 10 in `win10`) and the Microsoft Visual C++ runtime DLLs the program needs
  (`msvcp140*.dll`, `vcruntime140*.dll`), kept in that folder only: nothing is installed in
  Windows' own folders. A copy of this setup is kept
  there too, plus `install-manifest.json` (the record of everything this setup changed, used to
  undo it) and `setup.log`.
- **ii's own files** go to `%LOCALAPPDATA%\quickshell\ii`. They are replaced on every install,
  update and repair, so don't edit them.
- **Your settings** live in `%LOCALAPPDATA%\illogical-impulse` (`config.json` and friends). ii
  creates them as you change things; install and update never touch them.
- **ii's state and cache** (current colors, to-do list, notification history, logs, crash
  reports) go to `%LOCALAPPDATA%\quickshell` (`State`, `cache`, `run`), wallpaper thumbnails and
  the clipboard history's images to `%LOCALAPPDATA%\cache` (`thumbnails`, `quickshell`), and
  temporary files to `%TEMP%\quickshell`. A default color palette is put in place if there is
  none yet.
- **Start menu:** "illogical-impulse" (starts ii) and "illogical-impulse Settings".
- **Apps & features:** an entry called "illogical-impulse (ii-windows)". *Modify* opens this
  setup (update, repair, uninstall) and *Uninstall* opens it on the uninstall page.
- **The download:** the package comes from the latest release of
  [nunreasonable/ii-windows](https://github.com/nunreasonable/ii-windows) on GitHub and is
  checked against its published SHA-256 checksum before anything is installed.

Before the first install, the setup also writes down how Windows looks now, so uninstall can put
it back: your wallpaper (a copy of the picture is kept in `%LOCALAPPDATA%\ii-windows\restore`),
light or dark mode, the accent color and whether the taskbar hides automatically.

## The options

- **Start with Windows** (on by default): adds `illogical-impulse` to the "Run" list of your
  user (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`), so ii starts when you sign in.
- **Terminal setup** (on by default), the Windows version of ii's terminal look:
  - the *JetBrainsMono Nerd Font* is installed for your user (copied to
    `%LOCALAPPDATA%\Microsoft\Windows\Fonts` and registered under `HKCU`); font files you
    already have are left alone;
  - on Windows 10, if winget (App Installer) isn't there yet, a toggle in this option (on by
    default) downloads it and installs it first: see *winget* under "Windows 10" below;
  - *Oh My Posh*, *Starship* and *eza* are installed with `winget install --scope user`, unless
    they are already there; on Windows 10, *Windows Terminal* is installed the same way if it
    isn't already there (Windows 11 already has it, so nothing happens there). winget runs with
    `--accept-package-agreements` and `--accept-source-agreements`, which means this setup
    accepts those packages' licenses and the winget source terms for you. Only the tools this
    setup installed are remembered for uninstall;
  - a marked block is added to your PowerShell profile (`$PROFILE` of Windows PowerShell 5.1,
    and of PowerShell 7 if it is installed). It loads ii's prompt, its terminal colors and a few
    aliases (`ls` shows files with eza, `clear`, and `q` starts ii). The profile is copied to a
    backup before it's edited, and the block does nothing once ii is gone:

    ```
    # >>> illogical-impulse >>>
    # Loads illogical-impulse's terminal setup (prompt, colors, aliases). Remove this block, markers
    # included, to turn it off; the ii installer removes it on uninstall.
    $IiProfile = Join-Path $env:LOCALAPPDATA 'quickshell\ii\defaults\windows\terminal\profile.ps1'
    if (Test-Path -LiteralPath $IiProfile) { . $IiProfile }
    # <<< illogical-impulse <<<
    ```
- **Install PowerShell 7** (off by default): `winget install Microsoft.PowerShell`. It is
  installed for all users, so Windows asks for administrator permission.
- **Allow Windows PowerShell 5.1 to run profile scripts** (off by default): runs
  `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`. This is a **security setting**: it lets
  Windows PowerShell 5.1 run scripts stored on this computer, while scripts downloaded from the
  internet still need a signature. Without it, Windows PowerShell 5.1 doesn't load your profile,
  so ii's terminal look only appears in PowerShell 7, which allows this already. Nothing is
  changed if your policy already allows it, and uninstall puts the previous value back.

## What ii changes while it runs

- **Keyboard and mouse:** ii registers global shortcuts and installs a low-level keyboard hook
  and a low-level mouse hook. It takes over many Windows shortcuts: the Windows key pressed
  alone opens ii's search instead of the Start menu, and Win+Tab, Win+V, Win+A, Win+N,
  Win+1...0, Win+Q (closes the active window) and Win+D (maximizes) do ii things. Win+/ shows
  them all; you can change them in `%LOCALAPPDATA%\illogical-impulse\keybinds.json`. All of this
  stops when ii closes.
- **Taskbar:** by default the Windows taskbar is hidden and shows up when the pointer touches
  the bottom of the screen. For that, ii turns on the taskbar's auto-hide while it runs (if it
  was off) and turns it back off when it closes.
- **Wallpaper and colors:** on its first start ii uses your current Windows wallpaper without
  changing it. When you pick a wallpaper or switch light/dark mode in ii, it sets the same in
  Windows: the desktop wallpaper and Windows' light/dark mode. The Windows **accent color** ii
  changes whenever it loads its palette, including every time it starts: it becomes the main
  color of ii's palette.
- **Windows Terminal:** ii writes a color scheme matching its theme, plus font, cursor,
  padding and transparency settings for the PowerShell and Command Prompt profiles, as a Windows Terminal
  *fragment* in `%LOCALAPPDATA%\Microsoft\Windows Terminal\Fragments\illogical-impulse`. It does
  not edit Windows Terminal's `settings.json`; it only updates that file's date so open windows
  reload. Turning off terminal theming in ii's settings removes the fragment.
- **Clipboard:** ii keeps a history of what you copy (up to 200 items), opened with Win+V instead
  of Windows' own history. Text stays in memory only and is gone when ii closes; copied images
  are saved in `%LOCALAPPDATA%\cache\quickshell\clipboard` and deleted when ii closes. What apps
  mark to be left out of clipboard history (like passwords from password managers) isn't kept.
- **API keys:** if you save an API key in ii's AI panel, it is kept in Windows Credential
  Manager, in a credential called `illogical-impulse`.
- **Notifications:** ii reads the notifications Windows shows (through the notification access
  Windows gives apps) to show them in its own panel. Windows keeps showing its own banners too.
- **Virtual desktops:** ii's workspaces are Windows virtual desktops. Going to a workspace that
  doesn't exist yet creates a new desktop.
- What you change from ii's panels (volume and sound device, power mode, Bluetooth, Wi-Fi) are
  ordinary Windows settings, changed through Windows' own APIs. **Brightness** is a laptop
  screen's through Windows; on an external monitor ii changes the monitor's own brightness (over
  DDC/CI), and it stays that way. Where a monitor doesn't take DDC/CI (it's off in the monitor's
  menu, or it's a virtual machine's screen), ii dims the screen through its gamma instead, down to
  about half, and that goes back to normal when ii closes. ii's **night light** is not Windows'
  one: it adjusts the screen's gamma too, and everything goes back to normal when ii closes.

## Uninstall

Uninstall reads `install-manifest.json` and undoes what it lists:

- closes ii; removes the "Run" entry, the Start menu shortcuts and the Apps & features entry;
- takes the marked block out of your PowerShell profiles (a copy of each profile from before is
  saved in `%TEMP%`, and the uninstall tells you where);
- puts the PowerShell execution policy back, if this setup changed it;
- uninstalls with winget the terminal tools this setup installed (an option, on by default;
  PowerShell 7 too, if this setup installed it), then removes winget itself
  (`Remove-AppxPackage`) if this setup installed it on Windows 10, and removes the fonts it
  installed;
- deletes the Windows Terminal fragment folder, `%LOCALAPPDATA%\ii-windows`, ii's folders in
  `%LOCALAPPDATA%\quickshell` (`ii`, `State`, `cache`, `run`), `%LOCALAPPDATA%\cache\quickshell`
  and `%TEMP%\quickshell`, plus `%LOCALAPPDATA%\cache\thumbnails` if that folder wasn't there
  before the install;
- deletes your settings in `%LOCALAPPDATA%\illogical-impulse` and the `illogical-impulse`
  credential (the API keys), unless you tick *Keep my settings*;
- puts back the wallpaper, light/dark mode, accent color and taskbar auto-hide recorded before
  the first install, and makes sure the taskbar is visible (an option, on by default).

It can't undo: virtual desktops you or ii created; files you made (screenshots, recordings,
downloaded wallpapers); changes you made through ii to ordinary Windows settings (volume, sound
device, Bluetooth pairings, Wi-Fi, power mode) and to external monitors' brightness; a slideshow or Windows spotlight background
(the picture comes back, the slideshow doesn't). Tools and fonts you already had before
installing stay installed.

## Repair and update

**Repair** brings ii back to a basic stock configuration. It moves your settings
(`%LOCALAPPDATA%\illogical-impulse\*.json`) to a dated backup folder,
`%LOCALAPPDATA%\illogical-impulse\backups\repair-<date>`, so ii starts with its defaults. Then it
reinstalls the program files of the installed version (downloading them if needed), resets the
color palette to the default one, clears Quickshell's QML cache, applies the options you
installed with again, and restarts ii.

**Update** looks for a newer release on GitHub. If there is one, it downloads and checks it,
replaces the program files and ii's own files, keeps your settings, and restarts ii.

## Good to know

- This setup and ii's programs are **not signed**. Windows SmartScreen may say "Windows
  protected your PC"; *More info → Run anyway* starts it. Do that only if you got it from the
  link below.
- You need Windows 10 version 2004 or newer (build 19041; 22H2, build 19045, recommended) or
  Windows 11 (build 22000 or newer). winget (App Installer) is needed for the terminal tools.
- The setup's log is `%LOCALAPPDATA%\ii-windows\setup.log`. While it runs, the setup keeps its
  temporary files (the download, its window's browser data) in `%TEMP%\ii-windows-setup-...`
  and deletes them a few seconds after it closes.

## Windows 10

ii-windows also installs on Windows 10 version 2004 or newer (build 19041; 22H2, build 19045,
recommended), next to Windows 11. A few things work differently there:

- Below build 19041, the setup refuses to install: a check on the options page blocks it with a
  message saying it needs Windows 10 version 2004 or newer, 22H2 recommended, or Windows 11. From
  19041 up, the check passes and labels the system "Windows 10" with its version and build; from
  build 22000 up it's labelled "Windows 11", same as before.
- The setup needs the **Microsoft Edge WebView2 Runtime** to show its window at all. Windows 11
  always has it; if it's missing on Windows 10, the setup shows a message box, before anything
  else, offering to open Microsoft's WebView2 download page, then closes.
- **winget** (App Installer) isn't built into Windows 10 either; it usually arrives later through
  the Microsoft Store. If the *Terminal setup* option is on and winget can't be found, this
  setup can install it first, before the terminal tools below: a toggle in that option, on by
  default, downloads the App Installer package and its dependencies (about 300 MB together) from
  the latest release of [microsoft/winget-cli](https://github.com/microsoft/winget-cli) on
  GitHub, and installs it for your user with `Add-AppxPackage` - no administrator permission.
  Some winget-cli releases publish the msixbundle's SHA-256 next to it; when that's there, it's
  checked before installing, and this setup says so; when it isn't, this setup doesn't invent a
  check of its own, since Windows checks the package's own Microsoft signature on install anyway.
  Turning the toggle off skips this (same as before: the terminal tools are skipped too).
  Uninstall removes it with `Remove-AppxPackage`, only if this setup installed it. A failed
  download or install here is a warning, not a failed setup: the terminal tools are then skipped
  exactly as they are when winget was never found.
- **Windows Terminal** isn't built into Windows 10. If the *Terminal setup* option is on and
  `wt.exe` can't be found (and winget doesn't already list it installed), this setup installs it
  with `winget install --scope user Microsoft.WindowsTerminal`, the same way as Oh My Posh,
  Starship and eza; uninstall removes it only if this setup installed it. On Windows 11, where
  it's normally already there, this does nothing.

ii itself also differs a little on Windows 10:

- **Taskbar:** Windows 10 lets the taskbar sit on any edge of the screen. The hover-only taskbar
  follows it: it shows up when the pointer touches the edge the taskbar is on.
- **Virtual desktops:** Windows 10 gives ii no direct way to create or remove desktops. Going to a
  workspace that doesn't exist yet creates the desktop by pressing *Ctrl+Win+D* for you, as you
  would by hand, and ii can't remove desktops there.
- **Blur behind panels** uses Windows 10's older blur effect, whose rounded corners are a bit
  jagged.
- **Terminal:** the terminal shortcuts open Windows Terminal, or Windows PowerShell where Windows
  Terminal isn't installed. Without winget (App Installer, which not every Windows 10 has), the
  terminal tools, Windows Terminal and PowerShell 7 can't be installed and are skipped.

## Source code and licenses

- ii-windows (packaging and this setup):
  [github.com/nunreasonable/ii-windows](https://github.com/nunreasonable/ii-windows)
- Quickshell for Windows, branch `windows`:
  [github.com/nunreasonable/quickshell](https://github.com/nunreasonable/quickshell). Quickshell
  is licensed under the LGPL-3.0.
- illogical-impulse for Windows, branch `ii-windows`:
  [github.com/nunreasonable/dots-hyprland](https://github.com/nunreasonable/dots-hyprland).
  end-4's dots-hyprland is licensed under the GPL-3.0.
- Also included, among others: Qt 6 (LGPL-3.0), the FFmpeg libraries of Qt Multimedia
  (LGPL-2.1 or later), matugen
  (GPL-2.0), VirtualDesktopAccessor (MIT), the Microsoft Visual C++ runtime (redistributable
  files under Microsoft's license terms), the fonts JetBrainsMono Nerd Font, Rubik, Readex Pro,
  Space Grotesk and Google Sans Flex (SIL Open Font License), Material Symbols (Apache-2.0) and
  Adwaita icons (CC-BY-SA 3.0 / LGPL-3.0).
- This setup is built with Tauri (MIT / Apache-2.0) and shows text in Rubik (SIL Open Font
  License).
