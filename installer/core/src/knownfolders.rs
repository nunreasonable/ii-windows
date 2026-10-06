use std::io;
use std::path::Path;

use crate::progress::Msg;

pub const ERROR_CLOUD_FILE_PROVIDER_NOT_RUNNING: i32 = 362;

pub fn is_cloud_provider_not_running(e: &io::Error) -> bool {
	e.raw_os_error() == Some(ERROR_CLOUD_FILE_PROVIDER_NOT_RUNNING)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownFolder {
	Documents,
	Desktop,
	Pictures,
	Videos,
}

impl KnownFolder {
	pub const ALL: [KnownFolder; 4] =
		[KnownFolder::Documents, KnownFolder::Desktop, KnownFolder::Pictures, KnownFolder::Videos];

	pub fn check_id(&self) -> &'static str {
		match self {
			KnownFolder::Documents => "known_folder_documents",
			KnownFolder::Desktop => "known_folder_desktop",
			KnownFolder::Pictures => "known_folder_pictures",
			KnownFolder::Videos => "known_folder_videos",
		}
	}

	fn label(&self) -> &'static str {
		match self {
			KnownFolder::Documents => "Documents",
			KnownFolder::Desktop => "Desktop",
			KnownFolder::Pictures => "Pictures",
			KnownFolder::Videos => "Videos",
		}
	}

	fn breaks(&self) -> &'static str {
		match self {
			KnownFolder::Documents => {
				"the PowerShell profile this setup adds, and anything else that keeps files in Documents"
			}
			KnownFolder::Desktop => "shortcuts and files kept on the desktop",
			KnownFolder::Pictures => "ii's screenshots and its wallpaper picker",
			KnownFolder::Videos => "ii's screen recordings",
		}
	}

	fn msg_key(&self) -> &'static str {
		match self {
			KnownFolder::Documents => "known_folder_documents_blocked",
			KnownFolder::Desktop => "known_folder_desktop_blocked",
			KnownFolder::Pictures => "known_folder_pictures_blocked",
			KnownFolder::Videos => "known_folder_videos_blocked",
		}
	}
}

pub fn blocked_msg(folder: KnownFolder, path: &Path) -> Msg {
	let p = path.display().to_string();
	let text = format!(
		"Your {label} folder points to {p}, and Windows can't open it (for example, OneDrive was removed, but the folder still points into it). This breaks {breaks}. To fix it: right-click {label} in File Explorer, choose Properties, open the Location tab and select \"Restore Default\" (answer No if it offers to move your files there); or reinstall OneDrive first to get your files back, then try again.",
		label = folder.label(),
		breaks = folder.breaks(),
	);
	Msg::new(folder.msg_key(), text).with("path", p)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn detects_cloud_provider_error_by_code_only() {
		assert!(is_cloud_provider_not_running(&io::Error::from_raw_os_error(362)));
		assert!(!is_cloud_provider_not_running(&io::Error::from_raw_os_error(5)));
		assert!(!is_cloud_provider_not_running(&io::Error::new(io::ErrorKind::NotFound, "nope")));
	}

	#[test]
	fn blocked_message_names_the_folder_and_path() {
		let m = blocked_msg(KnownFolder::Documents, Path::new(r"C:\Users\u\OneDrive\Documents"));
		assert_eq!(m.key, "known_folder_documents_blocked");
		assert!(m.text.contains("Documents"));
		assert!(m.text.contains(r"C:\Users\u\OneDrive\Documents"));
		assert!(m.text.contains("PowerShell profile"));
		assert_eq!(m.params.get("path").map(String::as_str), Some(r"C:\Users\u\OneDrive\Documents"));
	}

	#[test]
	fn each_known_folder_has_its_own_key_and_breakage_text() {
		let mut keys = Vec::new();
		for folder in KnownFolder::ALL {
			let m = blocked_msg(folder, Path::new(r"C:\x"));
			assert!(m.key.starts_with("known_folder_"));
			assert!(m.key.ends_with("_blocked"));
			assert_eq!(m.key, format!("{}_blocked", folder.check_id()));
			keys.push(m.key);
		}
		let unique: std::collections::BTreeSet<_> = keys.iter().collect();
		assert_eq!(unique.len(), keys.len(), "check ids must be distinct: {keys:?}");
	}

	#[test]
	fn pictures_and_videos_name_their_own_ii_features() {
		let pictures = blocked_msg(KnownFolder::Pictures, Path::new(r"C:\x"));
		assert!(pictures.text.contains("screenshots"));
		assert!(pictures.text.contains("wallpaper"));
		let videos = blocked_msg(KnownFolder::Videos, Path::new(r"C:\x"));
		assert!(videos.text.contains("screen recordings"));
	}
}
