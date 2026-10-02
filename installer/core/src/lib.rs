//! Everything the ii-windows installer does, minus the window: the UI (app/) only calls in here.

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
pub const SHORTCUT_MAIN: &str = "illogical-impulse.lnk";
pub const SHORTCUT_SETTINGS: &str = "illogical-impulse Settings.lnk";
pub const DISPLAY_NAME: &str = "illogical-impulse (ii-windows)";
pub const PUBLISHER: &str = "nunreasonable";

/// Terminal tools installed per user through winget, with the command each one provides.
pub const TERMINAL_TOOLS: [(&str, &str); 3] =
	[("JanDeDobbeleer.OhMyPosh", "oh-my-posh"), ("Starship.Starship", "starship"), ("eza-community.eza", "eza")];
pub const PWSH_WINGET_ID: &str = "Microsoft.PowerShell";
