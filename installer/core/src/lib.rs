//! Everything the ii-windows installer does, minus the window: the UI (app/) only calls in here.

pub mod appinstaller;
pub mod fsops;
pub mod log;
pub mod manifest;
pub mod package;
pub mod paths;
pub mod profile;
pub mod progress;
pub mod readme;
pub mod release;
pub mod ttf;
pub mod version;

#[cfg(windows)]
pub mod ops;
#[cfg(windows)]
pub mod win;

pub const SETUP_EXE: &str = "ii-windows-setup.exe";
pub const MANIFEST: &str = "install-manifest.json";
pub const LOG: &str = "setup.log";
/// HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\<this>
pub const UNINSTALL_KEY_NAME: &str = "ii-windows";
/// HKCU\Software\Microsoft\Windows\CurrentVersion\Run value name.
pub const RUN_VALUE: &str = "illogical-impulse";
/// The Credential Manager entry ii keeps API keys in (KeyringStorage.qml).
pub const CREDENTIAL_TARGET: &str = "illogical-impulse";
pub const SHORTCUT_MAIN: &str = "illogical-impulse.lnk";
pub const SHORTCUT_SETTINGS: &str = "illogical-impulse Settings.lnk";
pub const DISPLAY_NAME: &str = "illogical-impulse (ii-windows)";
pub const PUBLISHER: &str = "nunreasonable";

/// Terminal tools installed per user through winget, with the command each one provides.
/// Windows Terminal is last: Windows 11 normally already has it, so the generic "already
/// installed" check in `ops::install_tools` is what skips it there; Windows 10 doesn't have it
/// built in, so this is what installs it.
pub const TERMINAL_TOOLS: [(&str, &str); 4] = [
	("JanDeDobbeleer.OhMyPosh", "oh-my-posh"),
	("Starship.Starship", "starship"),
	("eza-community.eza", "eza"),
	("Microsoft.WindowsTerminal", "wt"),
];
pub const PWSH_WINGET_ID: &str = "Microsoft.PowerShell";
/// App Installer's Appx package name (not its package family name below) - used to remove it
/// with `Get-AppxPackage -Name ... | Remove-AppxPackage` on uninstall.
pub const APP_INSTALLER_NAME: &str = "Microsoft.DesktopAppInstaller";
/// App Installer's package family name, for the install record and the README. Windows 10 may
/// not have it; winget is part of it.
pub const APP_INSTALLER_PACKAGE_FAMILY: &str = "Microsoft.DesktopAppInstaller_8wekyb3d8bbwe";
