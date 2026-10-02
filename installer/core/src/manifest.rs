//! `%LOCALAPPDATA%\ii-windows\install-manifest.json`: what the installer did and what Windows
//! looked like before, so uninstall can undo exactly that and nothing else.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Options {
	/// HKCU Run value starting ii at sign-in.
	pub autostart: bool,
	/// Fonts + Oh My Posh/Starship/eza + the profile block.
	pub terminal: bool,
	/// PowerShell 7 through winget (machine-wide, UAC).
	pub pwsh7: bool,
	/// Set-ExecutionPolicy -Scope CurrentUser RemoteSigned for Windows PowerShell 5.1.
	pub exec_policy: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RunValue {
	pub name: String,
	/// What the value held before the installer wrote it (restored instead of deleted).
	pub previous: Option<String>,
	pub set: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Font {
	pub file: PathBuf,
	pub registry_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct WingetPackage {
	pub id: String,
	pub installed_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ProfileEdit {
	/// "powershell" (Windows PowerShell 5.1) or "pwsh" (PowerShell 7).
	pub shell: String,
	pub path: PathBuf,
	/// The profile file didn't exist and the installer created it (and maybe its folder).
	pub created_file: bool,
	pub created_dir: Option<PathBuf>,
	/// The block was added by the installer (false: it was already there).
	pub block_added: bool,
	pub backup: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ExecPolicyChange {
	pub changed: bool,
	/// The CurrentUser scope value before ("Undefined" when it had none).
	pub previous: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Items {
	pub shortcuts: Vec<PathBuf>,
	pub uninstall_key: Option<String>,
	pub run_value: Option<RunValue>,
	pub fonts: Vec<Font>,
	pub winget: Vec<WingetPackage>,
	pub profiles: Vec<ProfileEdit>,
	pub exec_policy: Option<ExecPolicyChange>,
	pub colors_seeded: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct MonitorWallpaper {
	pub monitor: String,
	pub path: String,
	/// Copy of the picture taken at install time, relative to the install dir. Windows keeps
	/// only a transcoded cache of some wallpapers, and that cache is overwritten when ii sets a
	/// new one, so the original can't be found again later.
	pub backup: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct WallpaperState {
	pub monitors: Vec<MonitorWallpaper>,
	pub position: Option<i32>,
	pub background_color: Option<u32>,
	pub slideshow: bool,
	/// HKCU\...\Explorer\Wallpapers BackgroundType: 0 picture, 1 solid color, 2 slideshow, 3 spotlight.
	pub background_type: Option<u32>,
	/// HKCU\Control Panel\Desktop WallPaper.
	pub registry_path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PreInstall {
	pub recorded_at: String,
	pub taskbar_autohide: Option<bool>,
	pub apps_use_light_theme: Option<u32>,
	pub system_uses_light_theme: Option<u32>,
	pub accent_color: Option<u32>,
	pub colorization_color: Option<u32>,
	pub colorization_afterglow: Option<u32>,
	pub wallpaper: Option<WallpaperState>,
	/// Whether %LOCALAPPDATA%\cache and its `thumbnails` folder were there before ii, so
	/// uninstall only removes them if ii made them (None: not recorded, leave them).
	pub generic_cache_existed: Option<bool>,
	pub thumbnails_existed: Option<bool>,
	/// ii was running from somewhere else while this was recorded, so the taskbar values may be
	/// the ones ii sets while it runs rather than the user's own.
	pub other_instance_running: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Manifest {
	pub schema: u32,
	pub product: String,
	pub version: String,
	pub setup_version: String,
	/// "installing" while an install is in progress, "installed" after.
	pub state: String,
	pub source: String,
	pub installed_at: String,
	pub updated_at: Option<String>,
	pub repaired_at: Option<String>,
	pub install_dir: PathBuf,
	pub options: Options,
	pub items: Items,
	pub pre_install: Option<PreInstall>,
}

impl Manifest {
	pub fn new(install_dir: &Path) -> Manifest {
		Manifest {
			schema: SCHEMA,
			product: "ii-windows".into(),
			state: "installing".into(),
			install_dir: install_dir.to_path_buf(),
			..Default::default()
		}
	}

	pub fn load(path: &Path) -> std::io::Result<Option<Manifest>> {
		match std::fs::read(path) {
			Ok(bytes) => serde_json::from_slice(&bytes)
				.map(Some)
				.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
			Err(e) => Err(e),
		}
	}

	/// Written to a temp file first and renamed over the old one, so a crash mid-write never
	/// leaves a truncated manifest behind.
	pub fn save(&self, path: &Path) -> std::io::Result<()> {
		if let Some(dir) = path.parent() {
			std::fs::create_dir_all(dir)?;
		}
		let tmp = path.with_extension("json.tmp");
		let json = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
		std::fs::write(&tmp, json)?;
		std::fs::rename(&tmp, path)
	}

	pub fn has_winget(&self, id: &str) -> bool {
		self.items.winget.iter().any(|p| p.id.eq_ignore_ascii_case(id))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn roundtrip_and_unknown_fields() {
		let dir = std::env::temp_dir().join(format!("iiw-manifest-test-{}", std::process::id()));
		let path = dir.join("install-manifest.json");
		let mut m = Manifest::new(&dir);
		m.version = "0.1.0".into();
		m.options.autostart = true;
		m.items.fonts.push(Font { file: "C:\\x.ttf".into(), registry_name: "X (TrueType)".into() });
		m.pre_install = Some(PreInstall { taskbar_autohide: Some(false), ..Default::default() });
		m.save(&path).unwrap();
		let back = Manifest::load(&path).unwrap().unwrap();
		assert_eq!(back, m);
		// Older/newer installers may add fields; loading must not fail on them.
		std::fs::write(&path, br#"{"version":"0.2.0","future":{"x":1},"options":{"autostart":true}}"#).unwrap();
		let partial = Manifest::load(&path).unwrap().unwrap();
		assert_eq!(partial.version, "0.2.0");
		assert!(partial.options.autostart);
		assert!(Manifest::load(&dir.join("missing.json")).unwrap().is_none());
		std::fs::remove_dir_all(&dir).unwrap();
	}
}
