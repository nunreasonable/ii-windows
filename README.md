<div align="center">
    <picture>
        <source media="(prefers-color-scheme: dark)" srcset="assets/illogical-impulse-for-windows-transparent.png">
        <img src="assets/illogical-impulse-for-windows.png" alt="illogical-impulse for Windows" width="560">
    </picture>
    <h3>【 end_4's illogical-impulse, ported to Windows 10 and 11 】</h3>
</div>

<div align="center">

![](https://img.shields.io/github/v/release/nunreasonable/ii-windows?style=for-the-badge&logo=github&color=8ad7eb&logoColor=D9E0EE&labelColor=1E202B)
![](https://img.shields.io/github/downloads/nunreasonable/ii-windows/total?style=for-the-badge&logo=windows11&color=86dbd7&logoColor=D9E0EE&labelColor=1E202B)
![](https://img.shields.io/github/last-commit/nunreasonable/ii-windows?style=for-the-badge&logo=git&color=86dbce&logoColor=D9E0EE&labelColor=1E202B)

</div>

<div align="center">
    <h2>• overview •</h2>
    <h3></h3>
</div>

> [!WARNING]
> This is an **unofficial** port. It isn't made or supported by end-4 or by the Quickshell
> developers, so please don't take Windows problems to their repos: open an issue here instead.
> It's tested on Windows 10 and 11, on virtual machines and on real hardware.
>
> For questions and help, there's an [ii-windows thread](https://discord.com/channels/1393080422245863506/1560371137123188738)
> on the illogical-impulse Discord server. Bugs and feature requests go in an issue here.

<details>
  <summary>What this is/isn't</summary>

  - [illogical-impulse](https://github.com/end-4/dots-hyprland) (ii), end-4's Quickshell desktop shell for Hyprland, running natively on Windows 10 and 11
  - Bar with the system tray, sidebars, a Spotlight-style search, notifications, cheatsheet (with a System tab for CPU, GPU, memory and disks), settings app, screen recording, translation and Material You colors from your wallpaper, the same QML as on Linux wherever it could stay the same
  - Two looks to pick from, with the same features and options: ii's original style or the one from pctrade's [end4-pC](https://github.com/pctrade/end4-pC) fork (its bar layout, settings card or dashboard, sidebar banner, workspace numbers, a color picker on the bar)
  - Everything end4-pC adds, in both looks: a bar you can rearrange (with a dynamic island and a screen frame) in seven styles (Hug, Float, Rect, M3, M3 Hug, Panel, Split Hug) with an OLED black option, a dock on any screen edge with a right-click menu (or pinned apps inside the bar), more desktop widgets (weather with forecast, calendar, notes, to-do, timers, world clock, media with lyrics, a pixel clock, visualizer styles, images, stickers), wallpaper transitions, a centered or split-blur wallpaper, collages of several pictures, Wallhaven, Unsplash, Pexels and more in the wallpaper selector, a media card and reorderable sections in the sidebar, a 10-band equalizer, and presets you can export, import and share
  - A System page in Settings for the Windows side of what end4-pC's Hyprland page does: displays (resolution, refresh rate, position, scale, HDR, with a keep-or-revert countdown), keyboard, mouse, touchpad and cursor, window borders and corners (Windows 11), tiling layouts (dwindle or master), screen-off and sleep times, and the apps that start with Windows or with ii
  - It lives next to Explorer: the Windows taskbar is still there (hidden until you touch the screen edge it sits on, or back in place of ii's bar with Ctrl+Super+P), and ii's widgets (clock, weather, CPU/RAM/disk) sit on the Windows desktop next to your icons, over the Windows wallpaper, which ii keeps the same as the one you pick
  - Live wallpapers: pick a video (MP4, WebM, MKV, AVI, MOV) like any picture and it plays behind the desktop icons, with the colors taken from one of its frames. It pauses when a window covers the desktop, the screen is locked or game mode is on
  - A Now Playing pill: when the song changes, the cover flies out of the bar into a floating pill with the title, artists and album, which you can drag anywhere
  - NOT a Windows replacement: windows are still Windows' own and ii's workspaces are Windows' virtual desktops. Optional tiling (Hyprland's dwindle, off by default) works on top of them, with no extra window manager to install

</details>

<details>
  <summary>How the port works</summary>

  - **Quickshell for Windows**: a fork of Quickshell with a Windows backend. Panels are AppBars, input masks come from a low-level mouse hook, global shortcuts from `RegisterHotKey` plus a keyboard hook (for the lone Windows key), and the `Quickshell.Hyprland` module is backed by a window tracker and Windows' virtual desktops
  - **Tiling**: an optional dwindle layout in the Windows backend, one per virtual desktop and monitor, driven by the same dispatchers ii's keybinds already use (`togglefloating`, `togglesplit`, `movewindow`, `splitratio`...)
  - **Native services**: audio on Core Audio, media on WinRT media controls, notifications from Windows' own toasts, the system tray shared with Explorer's, screen recording on Windows.Graphics.Capture and Media Foundation, the media visualizer on a WASAPI loopback, clipboard history, Wi-Fi on WlanAPI, Bluetooth on WinRT, OCR on `Windows.Media.Ocr`, night light and software brightness as a gamma ramp, and so on, behind the same QML APIs ii already uses
  - **Helpers**: [SongRec](https://github.com/marin-m/SongRec)'s recognizer for music recognition and [MicroTeX](https://github.com/NanoMichael/MicroTeX) for LaTeX in the AI chat, both built for Windows
  - **ii for Windows**: a fork of ii where the services keep their public properties and swap only the implementation, so most modules are untouched
  - **Equalizer**: the curve goes to [Equalizer APO](https://sourceforge.net/projects/equalizerapo/) (free, installed separately) through a file it includes; Realtek Audio Console, MaxxAudio, Dolby, DTS, Nahimic and other sound apps have no public API, so ii only opens them
  - **Live wallpapers**: Media Foundation decodes the video on the GPU in its own thread and hands frames to Qt through a shared D3D11 texture; one decoder per file for all monitors. Without a GPU it plays at up to 720p and 15 fps
  - **Colors**: `matugen.exe` with ii's own template, so the palette from a wallpaper is the same as on Linux. Windows' wallpaper and light/dark mode follow what you pick in ii, and its accent color follows the palette
  - Quickshell (based on upstream 0.3.1), matugen, the helpers and the setup are cross-compiled from Linux (clang-cl + xwin, Qt 6.11) and tested in a Windows 11 VM (with and without a passed-through GPU) and a Windows 10 22H2 VM

</details>

<details>
  <summary>Installation</summary>

  - Download **`ii-windows-setup.exe`** from the [latest release](https://github.com/nunreasonable/ii-windows/releases/latest) and run it
    - It shows a README with everything it changes on your PC, and you have to read it before installing
    - It lets you pick the look (ii's original or end4-pC), with a picture of each; you can switch later in *Settings → Quick → Style*
    - Everything goes into your Windows user, no administrator needed (except the optional PowerShell 7)
    - It downloads the package from the release and checks its SHA-256 before installing anything
  - Update, repair and uninstall are in the same setup: *Settings → Apps → Installed apps → illogical-impulse (ii-windows) → Modify*
  - Every time ii starts, it checks GitHub for a new version and, if there is one, shows a notification with an *Update* button that opens the setup. It never updates by itself: nothing changes until you confirm in the setup. Beta installs are told about newer betas too. You can turn the notification off in *Settings → About*
  - The setup isn't signed yet, so SmartScreen may complain: *More info → Run anyway*
  - **Keybinds**: the same as on Linux where Windows allows it. Important ones:
    - `Win`+`/` = keybind list
    - `Ctrl`+`Win`+`/` (or the pencil on the keybind list) = keybind editor: turn a shortcut off to give it back to Windows (`Win` alone for the Start menu, `Win`+`V`, `Win`+`Shift`+`S`...) or change its keys. *Open file* there shows the `keybinds.json` behind it
    - `Win` alone = Spotlight search: apps, files, clipboard, emoji, system actions and the web (Settings → Services → Search switches back to ii's original overview)
    - `Win`+`Tab` = workspaces and their windows
    - `Win`+`Enter` = terminal
    - `Win`+drag = move a window, `Win`+right drag = resize it (like `bindm` on Hyprland)
  - Needs Windows 10 version 2004 or newer (22H2 recommended) or Windows 11 (build 22000 or
    newer). winget is needed for the optional terminal tools and FFmpeg; on Windows 10 without it, the setup can install it
  - Every time it opens, the setup checks GitHub for a newer release and updates itself first

</details>

<details>
  <summary>What works</summary>

  - **Tested on Windows 11 25H2** (in the VMs): bar, system tray, sidebars, launcher and search, cheatsheet, settings app, notifications (including the ones other apps show), clipboard history, screen snip and OCR, screen recording, translation, LaTeX rendering, wallpaper selector and Material colors, the wallpaper behind the desktop icons, virtual desktops as workspaces, taskbar on hover, terminal theming, weather, the media visualizer, the color picker, the Spotlight search, the cheatsheet's System tab, and the setup (install, update, self-update, repair, uninstall)
  - **Tested on Windows 10 22H2** (in a VM): the shell, virtual desktops, tiling, taskbar on hover, blur, the system tray (icons from apps started before ii included), the wallpaper behind the desktop icons with draggable widgets, the screen translator, the classic console's prompt and colors, software brightness, `Win`+drag to move and resize windows (also across monitors with different scaling, with and without tiling), the bar at 125% and 150% scaling and after changing the scale while ii runs, and the setup with Windows Terminal, the terminal tools and FFmpeg
  - **Settings app**: almost every option is there now, not only in `config.json`, in two layouts (ii's and end4-pC's) with exactly the same options
  - **Startup**: on a VM held to two slow cores at 40% and a disk at 120 IOPS, the bar shows in about 1.6 seconds (about 43 in 0.5.0)
  - **Also tested on real hardware**: an Intel i3-13100 with its integrated GPU only, on Windows 11
  - **Tested on real hardware**: Wi-Fi and Bluetooth
  - **Built on Windows' own APIs but barely tested**: audio, media controls, battery, laptop brightness and music recognition (the VMs have no battery or laptop screen, and no song was played to recognize), game mode, the Cloudflare WARP toggle (needs Cloudflare's WARP client) and the warning before shutting down while winget is installing
  - **Not on Windows**: ii's lock screen (`Win`+`L` uses Windows' own, which is the right one there) and the EasyEffects toggle
  - **Known limit**: while a window of an app running as administrator (Task Manager, installers) has the focus, Windows doesn't let ii see the keyboard, so ii's shortcuts don't work there and `Win`+`Q` opens Windows' search instead. Close those windows with `Alt`+`F4` or a middle click in the overview

</details>

<details>
  <summary>Software overview</summary>

  | Software | Purpose |
  | ------------- | ------------- |
  | [Quickshell for Windows](https://github.com/nunreasonable/quickshell/tree/windows) | The widget system, with a native Windows backend |
  | [ii for Windows](https://github.com/nunreasonable/dots-hyprland/tree/ii-windows) | The shell itself: bar, sidebars, settings and the rest |
  | [matugen](https://github.com/InioX/matugen) | Material You palette from the wallpaper |
  | [VirtualDesktopAccessor](https://github.com/Ciantic/VirtualDesktopAccessor) | Switching and creating Windows virtual desktops |
  | [SongRec](https://github.com/marin-m/SongRec) | Music recognition (its recognizer, built for Windows) |
  | [MicroTeX](https://github.com/NanoMichael/MicroTeX) | LaTeX formulas in the AI chat |
  | [FFmpeg](https://ffmpeg.org) | Optional, installed by the setup with winget if you want it |
  | [Oh My Posh](https://ohmyposh.dev), [Starship](https://starship.rs), [eza](https://github.com/eza-community/eza) | Optional terminal look, installed by the setup with winget |
  | [Tauri](https://tauri.app) | The setup's window |

</details>

<div align="center">
    <h2>• screenshots •</h2>
    <h3></h3>
</div>

<p align="center">
    <img src="assets/system.webp" alt="ii's cheatsheet on the System tab, running on a real PC with an RTX 5060 Ti">
    <br><sub>On a real PC: Windows 11 26H2 with an RTX 5060 Ti, the cheatsheet's System tab</sub>
</p>

<details>
<summary><b>Sensitive content (not explicit):</b> the desktop on the same PC, click to show</summary>
<p align="center">
    <img src="assets/pc-desktop.webp" alt="ii's bar and desktop clock on Windows 11, over an anime wallpaper of two characters in swimsuits">
</p>
</details>

| Sidebar, colors from the Windows wallpaper | Cheatsheet (`Win`+`/`) |
|:---|:---------------|
| <img src="assets/sidebar.webp" alt="ii's right sidebar on Windows 11"> | <img src="assets/cheatsheet.webp" alt="ii's keybind cheatsheet on Windows 11"> |
| **Windows Terminal in ii's colors** | **The setup** |
| <img src="assets/terminal.webp" alt="Windows Terminal with ii's color scheme and prompt"> | <img src="assets/setup.webp" alt="The ii-windows setup showing its README"> |

<div align="center">
    <h2>• privacy •</h2>
    <h3></h3>
</div>

ii-windows has no telemetry and sends nothing to its developers. A few things go online on their own (the update checks) and some features talk to outside services when you use them: [PRIVACY.md](PRIVACY.md) lists every one, with what is sent and when.

<div align="center">
    <h2>• Code signing policy •</h2>
    <h3></h3>
</div>

Starting with 1.1.0, `qs.exe`, `qsw.exe` and `ii-windows-setup.exe` in the releases are signed. Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org).

They are built from this repository by [GitHub Actions](.github/workflows/build.yml), from the source commits pinned in [sources.lock](sources.lock), and every signing request is approved by hand. The third-party programs in the package (matugen, SongRec, MicroTeX, VirtualDesktopAccessor, Qt) keep their own signatures or none.

- Committers and reviewers: [nunreasonable](https://github.com/nunreasonable)
- Approvers: [nunreasonable](https://github.com/nunreasonable)
- Privacy policy: [PRIVACY.md](PRIVACY.md)

<div align="center">
    <h2>• credits •</h2>
    <h3></h3>
</div>

All of the design and nearly all of the shell are other people's work. This repo only makes it run on Windows.

 - [@end-4](https://github.com/end-4) for illogical-impulse and [dots-hyprland](https://github.com/end-4/dots-hyprland): every widget here is theirs. If you like it, [support them](https://github.com/sponsors/end-4) and give the original a star
 - Everyone thanked in [dots-hyprland's README](https://github.com/end-4/dots-hyprland#readme), including [@clsty](https://github.com/clsty) and [@midn8hustlr](https://github.com/midn8hustlr), and all of [dots-hyprland's contributors](https://github.com/end-4/dots-hyprland/graphs/contributors)
 - [@pctrade](https://github.com/pctrade) for [end4-pC](https://github.com/pctrade/end4-pC), the fork of ii whose look and settings window are the second visual style here
 - [@outfoxxed](https://github.com/outfoxxed) and the contributors of [Quickshell](https://quickshell.outfoxxed.me), the toolkit underneath it all
 - [minimal-05/quickshell-macos](https://github.com/minimal-05/quickshell-macos), whose macOS port of Quickshell and ii showed this could be done
 - [InioX](https://github.com/InioX) for [matugen](https://github.com/InioX/matugen), [Ciantic](https://github.com/Ciantic) for [VirtualDesktopAccessor](https://github.com/Ciantic/VirtualDesktopAccessor), and Google's [material-color-utilities](https://github.com/material-foundation/material-color-utilities), ported for the terminal colors
 - [marin-m](https://github.com/marin-m) for [SongRec](https://github.com/marin-m/SongRec) and [NanoMichael](https://github.com/NanoMichael) for [MicroTeX](https://github.com/NanoMichael/MicroTeX)
 - [RetroBar](https://github.com/dremin/RetroBar) and [ManagedShell](https://github.com/cairoshell/ManagedShell), whose way of sharing the tray with Explorer the Windows tray follows
 - The Qt Project, and the fonts and icons ii ships with: JetBrains Mono and Nerd Fonts, Readex Pro, Space Grotesk, Google Sans Flex, Material Symbols and GNOME's Adwaita icons (the setup itself uses Rubik)

<div align="center">
    <h2>• license •</h2>
    <h3></h3>
</div>

- This repo (the setup and the packaging scripts) is GPL-3.0, like ii. See [LICENSE](LICENSE)
- The forks keep their upstream licenses: Quickshell for Windows is LGPL-3.0 and ii for Windows is GPL-3.0
- The release package also includes Qt (LGPL-3.0), matugen (GPL-2.0), VirtualDesktopAccessor (MIT), SongRec's recognizer (GPL-3.0-or-later, source in [tools/songrec](tools/songrec)), MicroTeX (MIT, build in [tools/microtex](tools/microtex)), the Visual C++ runtime and fonts under the SIL Open Font License and Apache-2.0. [THIRD-PARTY.md](THIRD-PARTY.md) lists them all with their versions and sources, and the license texts are in the package's `licenses` folder
- Copying: go ahead, just follow the licenses, like upstream asks
