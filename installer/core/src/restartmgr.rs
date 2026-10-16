#[cfg(windows)]
use std::path::Path;

pub const MAX_APPS: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockingApp {
	pub app_name: String,
	pub exe: String,
	pub pid: u32,
}

fn is_ii_exe(exe: &str) -> bool {
	exe.eq_ignore_ascii_case("qsw.exe") || exe.eq_ignore_ascii_case("qs.exe")
}

fn label(app: &LockingApp) -> String {
	match (app.app_name.is_empty(), app.exe.is_empty()) {
		(false, false) => format!("{} ({})", app.app_name, app.exe),
		(false, true) => app.app_name.clone(),
		(true, false) => app.exe.clone(),
		(true, true) => String::new(),
	}
}

fn normalize(mut apps: Vec<LockingApp>) -> Vec<LockingApp> {
	for app in &mut apps {
		app.app_name = app.app_name.trim().to_string();
		if is_ii_exe(&app.exe) {
			app.app_name = "illogical-impulse".into();
		}
	}
	let mut seen = std::collections::HashSet::new();
	apps.retain(|app| {
		!(app.app_name.is_empty() && app.exe.is_empty())
			&& seen.insert((app.app_name.to_lowercase(), app.exe.to_lowercase()))
	});
	apps.truncate(MAX_APPS);
	apps
}

pub fn describe(apps: &[LockingApp]) -> Option<String> {
	let apps = normalize(apps.to_vec());
	if apps.is_empty() {
		return None;
	}
	let text = apps.iter().map(label).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(", ");
	(!text.is_empty()).then_some(text)
}

#[cfg(windows)]
mod platform {
	use std::path::Path;
	use std::time::Duration;

	use windows::core::PCWSTR;
	use windows::Win32::Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS};
	use windows::Win32::System::RestartManager::{
		RmEndSession, RmGetList, RmRegisterResources, RmStartSession, CCH_RM_SESSION_KEY, RM_PROCESS_INFO,
	};

	use super::LockingApp;

	const MAX_PATHS: usize = 16;
	const MAX_FILES: usize = 2048;
	const MAX_PROCESSES: u32 = 64;
	const MAX_RM_QUERY: u32 = 4096;

	fn wide(s: &std::ffi::OsStr) -> Vec<u16> {
		use std::os::windows::ffi::OsStrExt;
		s.encode_wide().chain(std::iter::once(0)).collect()
	}

	fn expand(paths: &[std::path::PathBuf]) -> Vec<std::path::PathBuf> {
		let explicit: std::collections::HashSet<&std::path::PathBuf> = paths.iter().collect();
		let mut files = Vec::new();
		let mut stack: Vec<std::path::PathBuf> = paths.to_vec();
		while let Some(path) = stack.pop() {
			if files.len() >= MAX_FILES {
				break;
			}
			match std::fs::symlink_metadata(&path) {
				Ok(meta) if meta.is_dir() => {
					if let Ok(entries) = std::fs::read_dir(&path) {
						stack.extend(entries.flatten().map(|e| e.path()));
					}
				}
				Ok(meta) if meta.is_file() && (explicit.contains(&path) || is_module(&path)) => files.push(path),
				_ => {}
			}
		}
		files
	}

	fn is_module(path: &std::path::Path) -> bool {
		path.extension()
			.and_then(|e| e.to_str())
			.is_some_and(|e| e.eq_ignore_ascii_case("exe") || e.eq_ignore_ascii_case("dll"))
	}

	fn query(paths: &[std::path::PathBuf]) -> Vec<LockingApp> {
		if paths.is_empty() {
			return Vec::new();
		}
		let wide_paths: Vec<Vec<u16>> = paths.iter().map(|p| wide(p.as_os_str())).collect();
		let ptrs: Vec<PCWSTR> = wide_paths.iter().map(|w| PCWSTR(w.as_ptr())).collect();
		let mut session: u32 = 0;
		let mut key_buf = [0u16; (CCH_RM_SESSION_KEY + 1) as usize];
		let key = windows::core::PWSTR(key_buf.as_mut_ptr());
		if unsafe { RmStartSession(&mut session, None, key) } != ERROR_SUCCESS {
			return Vec::new();
		}
		let apps = (|| -> Vec<LockingApp> {
			if unsafe { RmRegisterResources(session, Some(&ptrs), None, None) } != ERROR_SUCCESS {
				return Vec::new();
			}
			let mut needed: u32 = 0;
			let mut count: u32 = 0;
			let mut reason: u32 = 0;
			let status = unsafe { RmGetList(session, &mut needed, &mut count, None, &mut reason) };
			if status != ERROR_MORE_DATA || needed == 0 {
				return Vec::new();
			}
			let mut count = needed.min(MAX_RM_QUERY);
			let mut buf: Vec<RM_PROCESS_INFO> = vec![RM_PROCESS_INFO::default(); count as usize];
			let status = unsafe { RmGetList(session, &mut needed, &mut count, Some(buf.as_mut_ptr()), &mut reason) };
			if status != ERROR_SUCCESS {
				return Vec::new();
			}
			buf.truncate((count as usize).min(MAX_PROCESSES as usize));
			buf.into_iter()
				.filter_map(|info| {
					let pid = info.Process.dwProcessId;
					if pid == std::process::id() {
						return None;
					}
					let len = info.strAppName.iter().position(|&c| c == 0).unwrap_or(info.strAppName.len());
					let app_name = String::from_utf16_lossy(&info.strAppName[..len]);
					let exe = crate::win::image_path(pid)
						.and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
						.unwrap_or_default();
					Some(LockingApp { app_name, exe, pid })
				})
				.collect()
		})();
		let _ = unsafe { RmEndSession(session) };
		apps
	}

	pub fn locking(paths: &[&Path]) -> Vec<LockingApp> {
		if paths.is_empty() {
			return Vec::new();
		}
		let owned: Vec<std::path::PathBuf> = paths.iter().take(MAX_PATHS).map(|p| p.to_path_buf()).collect();
		let (tx, rx) = std::sync::mpsc::channel();
		std::thread::spawn(move || {
			let _ = tx.send(query(&expand(&owned)));
		});
		rx.recv_timeout(Duration::from_secs(8)).unwrap_or_default()
	}
}

#[cfg(windows)]
pub fn locking(paths: &[&Path]) -> Vec<LockingApp> {
	platform::locking(paths)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn app(name: &str, exe: &str) -> LockingApp {
		LockingApp { app_name: name.into(), exe: exe.into(), pid: 1 }
	}

	#[test]
	fn describe_empty_is_none() {
		assert_eq!(describe(&[]), None);
	}

	#[test]
	fn describe_formats_name_and_exe() {
		assert_eq!(describe(&[app("File Explorer", "explorer.exe")]), Some("File Explorer (explorer.exe)".into()));
	}

	#[test]
	fn describe_falls_back_to_exe_when_name_missing() {
		assert_eq!(describe(&[app("", "SearchIndexer.exe")]), Some("SearchIndexer.exe".into()));
	}

	#[test]
	fn describe_falls_back_to_name_when_exe_missing() {
		assert_eq!(describe(&[app("Something", "")]), Some("Something".into()));
	}

	#[test]
	fn describe_drops_entries_with_neither() {
		assert_eq!(describe(&[app("", "")]), None);
	}

	#[test]
	fn describe_renames_ii_processes() {
		assert_eq!(describe(&[app("Quickshell", "qsw.exe")]), Some("illogical-impulse (qsw.exe)".into()));
		assert_eq!(describe(&[app("", "qs.exe")]), Some("illogical-impulse (qs.exe)".into()));
	}

	#[test]
	fn describe_dedupes_same_app() {
		let apps =
			vec![app("Explorer", "explorer.exe"), app("explorer", "EXPLORER.EXE"), app("Explorer", "explorer.exe")];
		assert_eq!(describe(&apps), Some("Explorer (explorer.exe)".into()));
	}

	#[test]
	fn describe_caps_the_list() {
		let apps: Vec<LockingApp> = (0..8).map(|i| app(&format!("App{i}"), &format!("app{i}.exe"))).collect();
		let text = describe(&apps).unwrap();
		assert_eq!(text.split(", ").count(), MAX_APPS);
		assert!(text.contains("App0 (app0.exe)"));
		assert!(!text.contains("App7"));
	}

	#[test]
	fn describe_joins_with_comma_only_no_localized_word() {
		let text = describe(&[app("A", "a.exe"), app("B", "b.exe")]).unwrap();
		assert_eq!(text, "A (a.exe), B (b.exe)");
	}
}
