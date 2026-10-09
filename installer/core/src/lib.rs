pub mod appinstaller;
pub mod fsops;
pub mod knownfolders;
pub mod log;
pub mod manifest;
pub mod package;
pub mod paths;
pub mod profile;
pub mod progress;
pub mod readme;
pub mod release;
pub mod style;
pub mod ttf;
pub mod version;

#[cfg(windows)]
pub mod ops;
#[cfg(windows)]
pub mod win;

pub const SETUP_EXE: &str = "ii-windows-setup.exe";
pub const MANIFEST: &str = "install-manifest.json";
pub const LOG: &str = "setup.log";
pub const UNINSTALL_KEY_NAME: &str = "ii-windows";
pub const RUN_VALUE: &str = "illogical-impulse";
pub const CREDENTIAL_TARGET: &str = "illogical-impulse";
pub const SHORTCUT_MAIN: &str = "illogical-impulse.lnk";
pub const SHORTCUT_SETTINGS: &str = "illogical-impulse Settings.lnk";
pub const DISPLAY_NAME: &str = "illogical-impulse (ii-windows)";
pub const PUBLISHER: &str = "nunreasonable";

pub const TERMINAL_TOOLS: [(&str, &str); 4] = [
	("JanDeDobbeleer.OhMyPosh", "oh-my-posh"),
	("Starship.Starship", "starship"),
	("eza-community.eza", "eza"),
	("Microsoft.WindowsTerminal", "wt"),
];
pub const PWSH_WINGET_ID: &str = "Microsoft.PowerShell";
pub const FFMPEG_WINGET_ID: &str = "Gyan.FFmpeg";
pub const FFMPEG_CMD: &str = "ffmpeg";
pub const APP_INSTALLER_NAME: &str = "Microsoft.DesktopAppInstaller";
pub const APP_INSTALLER_PACKAGE_FAMILY: &str = "Microsoft.DesktopAppInstaller_8wekyb3d8bbwe";
