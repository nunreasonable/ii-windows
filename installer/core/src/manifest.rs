use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::arch::{self, Arch};
use crate::startup::{self, StartupMethod};

pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Options {
	pub autostart: bool,
	#[serde(deserialize_with = "startup::lenient_method")]
	pub autostart_method: Option<StartupMethod>,
	pub terminal: bool,
	pub pwsh7: bool,
	pub exec_policy: bool,
	pub ffmpeg: bool,
	pub visual_style: String,
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
pub struct StartupTask {
	pub name: String,
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
	pub startup_task: Option<StartupTask>,
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
	#[serde(deserialize_with = "arch::lenient", skip_serializing_if = "Option::is_none")]
	pub arch: Option<Arch>,
	#[serde(deserialize_with = "arch::lenient", skip_serializing_if = "Option::is_none")]
	pub wanted_arch: Option<Arch>,
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

	pub fn update_arch(&self, native: Arch) -> Arch {
		arch::follow(native, self.wanted_arch.or(self.arch))
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
		m.options.autostart_method = Some(StartupMethod::Run);
		m.items.startup_task = Some(StartupTask { name: "illogical-impulse-S-1-5-21-1".into() });
		m.items.fonts.push(Font { file: "C:\\x.ttf".into(), registry_name: "X (TrueType)".into() });
		m.pre_install = Some(PreInstall { taskbar_autohide: Some(false), ..Default::default() });
		m.save(&path).unwrap();
		let back = Manifest::load(&path).unwrap().unwrap();
		assert_eq!(back, m);
		std::fs::write(&path, br#"{"version":"0.2.0","future":{"x":1},"options":{"autostart":true}}"#).unwrap();
		let partial = Manifest::load(&path).unwrap().unwrap();
		assert_eq!(partial.version, "0.2.0");
		assert!(partial.options.autostart);
		assert_eq!(partial.options.autostart_method, None);
		std::fs::write(&path, br#"{"options":{"autostart":true,"autostart_method":"service"}}"#).unwrap();
		let future = Manifest::load(&path).unwrap().unwrap();
		assert!(future.options.autostart);
		assert_eq!(future.options.autostart_method, None);
		std::fs::write(&path, br#"{"options":{"autostart_method":"task"}}"#).unwrap();
		assert_eq!(Manifest::load(&path).unwrap().unwrap().options.autostart_method, Some(StartupMethod::Task));
		assert!(Manifest::load(&dir.join("missing.json")).unwrap().is_none());
		std::fs::remove_dir_all(&dir).unwrap();
	}

	#[test]
	fn old_manifest_means_x64() {
		let dir = std::env::temp_dir().join(format!("iiw-manifest-arch-{}", std::process::id()));
		let path = dir.join("install-manifest.json");
		std::fs::create_dir_all(&dir).unwrap();
		std::fs::write(&path, br#"{"schema":1,"version":"26.1.1.0","state":"installed"}"#).unwrap();
		let old = Manifest::load(&path).unwrap().unwrap();
		assert_eq!(old.arch, None);
		assert_eq!(old.arch.unwrap_or_default(), Arch::X64);
		assert_eq!(old.update_arch(Arch::Arm64), Arch::X64);
		assert_eq!(old.update_arch(Arch::X64), Arch::X64);
		old.save(&path).unwrap();
		let text = std::fs::read_to_string(&path).unwrap();
		assert!(!text.contains("arch"), "an untouched old manifest is written back without the new fields");

		let mut m = Manifest::new(&dir);
		m.arch = Some(Arch::Arm64);
		m.wanted_arch = Some(Arch::Arm64);
		m.save(&path).unwrap();
		let back = Manifest::load(&path).unwrap().unwrap();
		assert_eq!(back, m);
		assert!(std::fs::read_to_string(&path).unwrap().contains(r#""arch": "arm64""#));
		assert_eq!(back.update_arch(Arch::Arm64), Arch::Arm64);

		std::fs::write(&path, br#"{"version":"27.1.0.0","arch":"riscv64","wanted_arch":null}"#).unwrap();
		let future = Manifest::load(&path).unwrap().unwrap();
		assert_eq!(future.arch, None);
		assert_eq!(future.update_arch(Arch::Arm64), Arch::X64);
		std::fs::remove_dir_all(&dir).unwrap();
	}
}
