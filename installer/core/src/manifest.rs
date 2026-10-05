use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Options {
	pub autostart: bool,
	pub terminal: bool,
	pub pwsh7: bool,
	pub exec_policy: bool,
	pub ffmpeg: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RunValue {
	pub name: String,
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
	pub shell: String,
	pub path: PathBuf,
	pub created_file: bool,
	pub created_dir: Option<PathBuf>,
	pub block_added: bool,
	pub backup: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AppInstaller {
	pub installed_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ExecPolicyChange {
	pub changed: bool,
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
	pub app_installer: Option<AppInstaller>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct MonitorWallpaper {
	pub monitor: String,
	pub path: String,
	pub backup: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct WallpaperState {
	pub monitors: Vec<MonitorWallpaper>,
	pub position: Option<i32>,
	pub background_color: Option<u32>,
	pub slideshow: bool,
	pub background_type: Option<u32>,
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
	pub generic_cache_existed: Option<bool>,
	pub thumbnails_existed: Option<bool>,
	pub other_instance_running: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Manifest {
	pub schema: u32,
	pub product: String,
	pub version: String,
	pub setup_version: String,
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
		std::fs::write(&path, br#"{"version":"0.2.0","future":{"x":1},"options":{"autostart":true}}"#).unwrap();
		let partial = Manifest::load(&path).unwrap().unwrap();
		assert_eq!(partial.version, "0.2.0");
		assert!(partial.options.autostart);
		assert!(Manifest::load(&dir.join("missing.json")).unwrap().is_none());
		std::fs::remove_dir_all(&dir).unwrap();
	}
}
