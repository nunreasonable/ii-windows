<div align="center">
    <picture>
        <source media="(prefers-color-scheme: dark)" srcset="assets/illogical-impulse-for-windows-transparent.png">
        <img src="assets/illogical-impulse-for-windows.png" alt="illogical-impulse for Windows" width="560">
    </picture>
    <h3>【 end_4's illogical-impulse, ported to Windows 11 】</h3>
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
> It is experimental, expect rough edges.

<details>
  <summary>What this is/isn't</summary>

  - [illogical-impulse](https://github.com/end-4/dots-hyprland) (ii), end-4's Quickshell desktop shell for Hyprland, running natively on Windows 11
  - Bar, sidebars, launcher, notifications, cheatsheet, settings app and Material You colors from your wallpaper, the same QML as on Linux wherever it could stay the same
  - It lives next to Explorer: the Windows taskbar and tray are still there (hidden until you touch the bottom edge of the screen)
  - NOT a tiling window manager and NOT a Windows replacement: windows are still Windows' own, and ii's workspaces are Windows' virtual desktops

</details>

<details>
  <summary>How the port works</summary>

  - **Quickshell for Windows**: a fork of Quickshell with a Windows backend. Panels are AppBars, input masks come from a low-level mouse hook, global shortcuts from `RegisterHotKey` plus a keyboard hook (for the lone Windows key), and the `Quickshell.Hyprland` module is backed by a window tracker and Windows' virtual desktops
  - **Native services**: audio on Core Audio, media on WinRT media controls, notifications from Windows' own toasts, clipboard history, Wi-Fi on WlanAPI, Bluetooth on WinRT, OCR on `Windows.Media.Ocr`, night light as a gamma ramp, and so on, behind the same QML APIs ii already uses
  - **ii for Windows**: a fork of ii where the services keep their public properties and swap only the implementation, so most modules are untouched
  - **Colors**: `matugen.exe` with ii's own template, so the palette from a wallpaper is the same as on Linux. Windows' wallpaper and light/dark mode follow what you pick in ii, and its accent color follows the palette
  - Quickshell, matugen and the setup are cross-compiled from Linux (clang-cl + xwin, Qt 6.11) and tested in a Windows 11 VM with a passed-through GPU

</details>

<details>
  <summary>Installation</summary>

  - Download **`ii-windows-setup.exe`** from the [latest release](https://github.com/nunreasonable/ii-windows/releases/latest) and run it
    - It shows a README with everything it changes on your PC, and you have to read it before installing
    - Everything goes into your Windows user, no administrator needed (except the optional PowerShell 7)
    - It downloads the package from the release and checks its SHA-256 before installing anything
  - Update, repair and uninstall are in the same setup: *Settings → Apps → Installed apps → illogical-impulse (ii-windows) → Modify*
  - The setup isn't signed yet, so SmartScreen may complain: *More info → Run anyway*
  - **Keybinds**: the same as on Linux where Windows allows it. Important ones:
    - `Win`+`/` = keybind list
    - `Win` alone = search
    - `Win`+`Enter` = terminal
  - Needs Windows 11 (build 22000 or newer). winget is needed for the optional terminal tools

</details>

<details>
  <summary>What works</summary>

  - **Tested on Windows 11 25H2** (in the VM): bar, sidebars, launcher and search, cheatsheet, settings app, notifications (including the ones other apps show), clipboard history, screen snip and OCR, wallpaper selector and Material colors, virtual desktops as workspaces, taskbar on hover, terminal theming, and the setup (install, update, repair, uninstall)
  - **Built on Windows' own APIs but barely tested**: audio, media controls, Bluetooth, Wi-Fi, battery and brightness (the test VM has no Bluetooth, Wi-Fi, battery or laptop screen)
  - **Not on Windows (yet)**: ii's lock screen (`Win`+`L` uses Windows' own), translation, music recognition, LaTeX rendering, the EasyEffects and WARP toggles, and ii's system tray (the tray stays in Windows' taskbar). Screen recording needs `ffmpeg` on your PATH

</details>

<details>
  <summary>Software overview</summary>

  | Software | Purpose |
  | ------------- | ------------- |
  | [Quickshell for Windows](https://github.com/nunreasonable/quickshell/tree/windows) | The widget system, with a native Windows backend |
  | [ii for Windows](https://github.com/nunreasonable/dots-hyprland/tree/ii-windows) | The shell itself: bar, sidebars, settings and the rest |
  | [matugen](https://github.com/InioX/matugen) | Material You palette from the wallpaper |
  | [VirtualDesktopAccessor](https://github.com/Ciantic/VirtualDesktopAccessor) | Switching and creating Windows virtual desktops |
  | [Oh My Posh](https://ohmyposh.dev), [Starship](https://starship.rs), [eza](https://github.com/eza-community/eza) | Optional terminal look, installed by the setup with winget |
  | [Tauri](https://tauri.app) | The setup's window |

</details>

<div align="center">
    <h2>• screenshots •</h2>
    <h3></h3>
</div>

| Sidebar, colors from the Windows wallpaper | Cheatsheet (`Win`+`/`) |
|:---|:---------------|
| <img src="assets/sidebar.webp" alt="ii's right sidebar on Windows 11"> | <img src="assets/cheatsheet.webp" alt="ii's keybind cheatsheet on Windows 11"> |
| **Windows Terminal in ii's colors** | **The setup** |
| <img src="assets/terminal.webp" alt="Windows Terminal with ii's color scheme and prompt"> | <img src="assets/setup.webp" alt="The ii-windows setup showing its README"> |

<div align="center">
    <h2>• credits •</h2>
    <h3></h3>
</div>

All of the design and nearly all of the shell are other people's work. This repo only makes it run on Windows.

 - [@end-4](https://github.com/end-4) for illogical-impulse and [dots-hyprland](https://github.com/end-4/dots-hyprland): every widget here is theirs. If you like it, [support them](https://github.com/sponsors/end-4) and give the original a star
 - Everyone thanked in [dots-hyprland's README](https://github.com/end-4/dots-hyprland#readme), including [@clsty](https://github.com/clsty) and [@midn8hustlr](https://github.com/midn8hustlr), and all of [dots-hyprland's contributors](https://github.com/end-4/dots-hyprland/graphs/contributors)
 - [@outfoxxed](https://github.com/outfoxxed) and the contributors of [Quickshell](https://quickshell.outfoxxed.me), the toolkit underneath it all
 - [minimal-05/quickshell-macos](https://github.com/minimal-05/quickshell-macos), whose macOS port of Quickshell and ii showed this could be done
 - [InioX](https://github.com/InioX) for [matugen](https://github.com/InioX/matugen), [Ciantic](https://github.com/Ciantic) for [VirtualDesktopAccessor](https://github.com/Ciantic/VirtualDesktopAccessor), and Google's [material-color-utilities](https://github.com/material-foundation/material-color-utilities), ported for the terminal colors
 - The Qt Project, and the fonts and icons ii ships with: JetBrains Mono and Nerd Fonts, Rubik, Readex Pro, Space Grotesk, Google Sans Flex, Material Symbols and GNOME's Adwaita icons

<div align="center">
    <h2>• license •</h2>
    <h3></h3>
</div>

- Each part keeps its upstream license: Quickshell for Windows is LGPL-3.0 and ii for Windows is GPL-3.0, like the projects they are forked from
- The release package also includes Qt (LGPL-3.0), matugen (GPL-2.0), VirtualDesktopAccessor (MIT) and fonts under the SIL Open Font License and Apache-2.0. The setup's README lists them all
- Copying: go ahead, just follow the licenses, like upstream asks
