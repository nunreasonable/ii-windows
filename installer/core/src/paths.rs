//! Where everything lives. Built from the per-user roots so tests can point it at a temp dir.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize)]
pub struct Paths {
	/// %LOCALAPPDATA%
	pub local: PathBuf,
	/// %APPDATA%
	pub roaming: PathBuf,
	pub temp: PathBuf,
	/// The per-user Start menu "Programs" folder.
	pub start_menu: PathBuf,
	/// %LOCALAPPDATA%\ii-windows: program files + the installer's own files.
	pub install_dir: PathBuf,
	/// %LOCALAPPDATA%\quickshell
	pub quickshell: PathBuf,
	/// %LOCALAPPDATA%\quickshell\ii: the ii config `qsw.exe -c ii` loads.
	pub ii_config: PathBuf,
	/// %LOCALAPPDATA%\quickshell\State\user\generated\colors.json
	pub colors: PathBuf,
	/// %LOCALAPPDATA%\illogical-impulse: the user's own settings (config.json, ...).
	pub settings: PathBuf,
	/// %LOCALAPPDATA%\Microsoft\Windows\Fonts
	pub user_fonts: PathBuf,
	/// %LOCALAPPDATA%\Microsoft\Windows Terminal\Fragments\illogical-impulse
	pub wt_fragment: PathBuf,
}

impl Paths {
	pub fn from_roots(local: &Path, roaming: &Path, temp: &Path, start_menu: &Path) -> Paths {
		let quickshell = local.join("quickshell");
		Paths {
			local: local.into(),
			roaming: roaming.into(),
			temp: temp.into(),
			start_menu: start_menu.into(),
			install_dir: local.join("ii-windows"),
			ii_config: quickshell.join("ii"),
			colors: quickshell.join("State").join("user").join("generated").join("colors.json"),
			quickshell,
			settings: local.join("illogical-impulse"),
			user_fonts: local.join("Microsoft").join("Windows").join("Fonts"),
			wt_fragment: local.join("Microsoft").join("Windows Terminal").join("Fragments").join("illogical-impulse"),
		}
	}

	pub fn manifest(&self) -> PathBuf {
		self.install_dir.join(crate::MANIFEST)
	}
	pub fn log(&self) -> PathBuf {
		self.install_dir.join(crate::LOG)
	}
	pub fn setup_exe(&self) -> PathBuf {
		self.install_dir.join(crate::SETUP_EXE)
	}
	pub fn restore_dir(&self) -> PathBuf {
		self.install_dir.join("restore")
	}
	pub fn backup_dir(&self) -> PathBuf {
		self.install_dir.join("backup")
	}
	pub fn staging(&self) -> PathBuf {
		self.local.join("ii-windows.staging")
	}
	pub fn previous(&self) -> PathBuf {
		self.local.join("ii-windows.previous")
	}
	pub fn qs_exe(&self) -> PathBuf {
		self.install_dir.join("qs.exe")
	}
	pub fn qsw_exe(&self) -> PathBuf {
		self.install_dir.join("qsw.exe")
	}
	pub fn settings_qml(&self) -> PathBuf {
		self.ii_config.join("settings.qml")
	}
	pub fn default_colors(&self) -> PathBuf {
		self.ii_config.join("defaults").join("windows").join("colors.json")
	}
	/// Quickshell's own folders under %LOCALAPPDATA%\quickshell that ii's runs create.
	pub fn quickshell_dirs(&self) -> Vec<PathBuf> {
		["ii", "State", "cache", "run"].iter().map(|d| self.quickshell.join(d)).collect()
	}
	/// Qt's GenericCacheLocation (%LOCALAPPDATA%\cache): Quickshell keeps copied images there
	/// (`quickshell\clipboard`), and ii its wallpaper thumbnails (`thumbnails`).
	pub fn generic_cache(&self) -> PathBuf {
		self.local.join("cache")
	}
	/// ii's temp root (%TEMP%\quickshell, Directories.tempRoot).
	pub fn ii_temp(&self) -> PathBuf {
		self.temp.join("quickshell")
	}
}
