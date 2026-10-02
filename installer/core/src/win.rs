//! Thin wrappers over the Windows APIs the installer uses. Everything is per user (HKCU,
//! %LOCALAPPDATA%); the only machine-wide action, PowerShell 7, goes through winget and its UAC
//! prompt.

use std::ffi::OsStr;
use std::io::Read;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use windows::core::{Interface, GUID, HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, COLORREF, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{AddFontResourceW, RemoveFontResourceW};
use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
use windows::Win32::System::Com::{
	CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IPersistFile, CLSCTX_ALL, CLSCTX_INPROC_SERVER,
	COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
	CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
	OpenProcess, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject, PROCESS_NAME_WIN32,
	PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
};
use windows::Win32::UI::Shell::{
	DesktopWallpaper, FOLDERID_LocalAppData, FOLDERID_Programs, FOLDERID_RoamingAppData, IDesktopWallpaper,
	IShellLinkW, SHAppBarMessage, SHGetKnownFolderPath, ShellExecuteW, ShellLink, ABM_GETSTATE, ABM_SETSTATE,
	APPBARDATA, DESKTOP_WALLPAPER_POSITION, KF_FLAG_DEFAULT,
};
use windows::Win32::UI::WindowsAndMessaging::{
	EnumWindows, FindWindowW, GetClassNameW, SendMessageTimeoutW, ShowWindow, HWND_BROADCAST, SMTO_ABORTIFHUNG,
	SW_SHOWNA, SW_SHOWNORMAL, WM_FONTCHANGE, WM_SETTINGCHANGE,
};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use winreg::types::FromRegValue;
use winreg::RegKey;

use crate::manifest::{MonitorWallpaper, WallpaperState};
use crate::paths::Paths;

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const DETACHED_PROCESS: u32 = 0x0000_0008;
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
pub const UNINSTALL_ROOT: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";
pub const FONTS_KEY: &str = r"Software\Microsoft\Windows NT\CurrentVersion\Fonts";
pub const PERSONALIZE_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
pub const DWM_KEY: &str = r"Software\Microsoft\Windows\DWM";
pub const DESKTOP_KEY: &str = r"Control Panel\Desktop";
pub const WALLPAPERS_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Wallpapers";

fn wide(s: impl AsRef<OsStr>) -> Vec<u16> {
	s.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

fn pwstr_to_string(p: PWSTR) -> String {
	if p.is_null() {
		return String::new();
	}
	let s = unsafe { p.to_string().unwrap_or_default() };
	unsafe { CoTaskMemFree(Some(p.0 as *const _)) };
	s
}

// ---------------------------------------------------------------------------------------------
// COM, folders, system info

/// COM for the current thread (shortcuts, wallpaper). Uninitialized when dropped.
pub struct Com(bool);

impl Com {
	pub fn init() -> Com {
		let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
		Com(hr.is_ok())
	}
}

impl Drop for Com {
	fn drop(&mut self) {
		if self.0 {
			unsafe { CoUninitialize() };
		}
	}
}

fn known_folder(id: &GUID) -> Option<PathBuf> {
	let p = unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }.ok()?;
	let s = pwstr_to_string(p);
	(!s.is_empty()).then(|| PathBuf::from(s))
}

pub fn detect_paths() -> Paths {
	let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
	let local = known_folder(&FOLDERID_LocalAppData).or_else(|| env("LOCALAPPDATA")).unwrap_or_default();
	let roaming = known_folder(&FOLDERID_RoamingAppData).or_else(|| env("APPDATA")).unwrap_or_default();
	let start_menu =
		known_folder(&FOLDERID_Programs).unwrap_or_else(|| roaming.join(r"Microsoft\Windows\Start Menu\Programs"));
	Paths::from_roots(&local, &roaming, &std::env::temp_dir(), &start_menu)
}

pub fn windows_build() -> (u32, String) {
	let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
	let Ok(key) = key else { return (0, String::new()) };
	let build: String = key.get_value("CurrentBuildNumber").unwrap_or_default();
	let display: String = key.get_value("DisplayVersion").unwrap_or_default();
	(build.trim().parse().unwrap_or(0), display)
}

pub fn ui_language_is_portuguese() -> bool {
	let lang = unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() };
	// PRIMARYLANGID: low 10 bits. LANG_PORTUGUESE = 0x16.
	lang & 0x3ff == 0x16
}

pub fn free_space(path: &Path) -> Option<u64> {
	// The closest existing ancestor: the folder itself may not exist yet.
	let mut p = path.to_path_buf();
	while !p.exists() {
		p = p.parent()?.to_path_buf();
	}
	let w = wide(p.as_os_str());
	let mut free = 0u64;
	unsafe { GetDiskFreeSpaceExW(PCWSTR(w.as_ptr()), Some(&mut free), None, None) }.ok()?;
	Some(free)
}

// ---------------------------------------------------------------------------------------------
// Processes

#[derive(Debug, Clone, serde::Serialize)]
pub struct Proc {
	pub pid: u32,
	pub name: String,
	pub path: PathBuf,
}

/// Running processes whose image name is one of `names` (case-insensitive), with their full
/// image path when Windows lets us read it.
pub fn find_processes(names: &[&str]) -> Vec<Proc> {
	let mut out = Vec::new();
	let Ok(snap) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else { return out };
	let mut entry = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
	let mut ok = unsafe { Process32FirstW(snap, &mut entry) }.is_ok();
	while ok {
		let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
		let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
		if names.iter().any(|n| n.eq_ignore_ascii_case(&name)) {
			let path = image_path(entry.th32ProcessID).unwrap_or_default();
			out.push(Proc { pid: entry.th32ProcessID, name, path });
		}
		ok = unsafe { Process32NextW(snap, &mut entry) }.is_ok();
	}
	let _ = unsafe { CloseHandle(snap) };
	out
}

fn image_path(pid: u32) -> Option<PathBuf> {
	let h = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
	let mut buf = vec![0u16; 32768];
	let mut len = buf.len() as u32;
	let r = unsafe { QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len) };
	let _ = unsafe { CloseHandle(h) };
	r.ok()?;
	Some(PathBuf::from(std::ffi::OsString::from_wide(&buf[..len as usize])))
}

/// `true` once the process is gone (or was never there).
pub fn wait_exit(pid: u32, timeout: Duration) -> bool {
	let Ok(h) = (unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) }) else { return true };
	let r = unsafe { WaitForSingleObject(h, timeout.as_millis() as u32) };
	let _ = unsafe { CloseHandle(h) };
	r.0 == 0 // WAIT_OBJECT_0
}

pub fn terminate(pid: u32) -> bool {
	let Ok(h) = (unsafe { OpenProcess(PROCESS_TERMINATE | PROCESS_SYNCHRONIZE, false, pid) }) else { return false };
	let ok = unsafe { TerminateProcess(h, 1) }.is_ok();
	if ok {
		unsafe { WaitForSingleObject(h, 5000) };
	}
	let _ = unsafe { CloseHandle(h) };
	ok
}

/// Case-insensitive "is `path` inside `dir`".
pub fn path_in(path: &Path, dir: &Path) -> bool {
	let p = path.to_string_lossy().to_lowercase().replace('/', "\\");
	let mut d = dir.to_string_lossy().to_lowercase().replace('/', "\\");
	if !d.ends_with('\\') {
		d.push('\\');
	}
	p.starts_with(&d)
}

pub fn same_path(a: &Path, b: &Path) -> bool {
	let n = |p: &Path| p.to_string_lossy().to_lowercase().replace('/', "\\").trim_end_matches('\\').to_string();
	n(a) == n(b)
}

// ---------------------------------------------------------------------------------------------
// Child processes

#[derive(Debug, Clone)]
pub struct Output {
	pub code: Option<i32>,
	pub stdout: String,
	pub stderr: String,
	pub timed_out: bool,
}

impl Output {
	pub fn success(&self) -> bool {
		self.code == Some(0) && !self.timed_out
	}
	/// Both streams, with winget's spinner and progress-bar noise taken out, for the log.
	pub fn text(&self) -> String {
		let mut s = String::new();
		for part in [&self.stdout, &self.stderr] {
			for line in part.split(['\n', '\r']) {
				let l = line.trim();
				if l.is_empty() || l.chars().all(|c| "-\\|/ █▒░".contains(c)) {
					continue;
				}
				if l.contains('█') || l.contains('▒') {
					continue;
				}
				s.push_str(l);
				s.push('\n');
			}
		}
		s.trim_end().to_string()
	}
}

fn decode_output(bytes: &[u8]) -> String {
	let b = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
	String::from_utf8_lossy(b).replace('\u{8}', "")
}

/// Runs a program without a console window, stdin closed, and kills it if it outlives
/// `timeout` (only that child: it's ours).
pub fn run(exe: &Path, args: &[&str], timeout: Duration) -> std::io::Result<Output> {
	let mut child = Command::new(exe)
		.args(args)
		// A Process-scope policy (powershell -ExecutionPolicy Bypass) leaks to children through
		// this variable; what the setup asks PowerShell must be the user's own settings.
		.env_remove("PSExecutionPolicyPreference")
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.creation_flags(CREATE_NO_WINDOW)
		.spawn()?;
	let mut so = child.stdout.take().unwrap();
	let mut se = child.stderr.take().unwrap();
	let t1 = std::thread::spawn(move || {
		let mut v = Vec::new();
		let _ = so.read_to_end(&mut v);
		v
	});
	let t2 = std::thread::spawn(move || {
		let mut v = Vec::new();
		let _ = se.read_to_end(&mut v);
		v
	});
	let start = Instant::now();
	let mut timed_out = false;
	let status = loop {
		if let Some(s) = child.try_wait()? {
			break Some(s);
		}
		if start.elapsed() > timeout {
			timed_out = true;
			let _ = child.kill();
			break child.wait().ok();
		}
		std::thread::sleep(Duration::from_millis(100));
	};
	let stdout = decode_output(&t1.join().unwrap_or_default());
	let stderr = decode_output(&t2.join().unwrap_or_default());
	Ok(Output { code: status.and_then(|s| s.code()), stdout, stderr, timed_out })
}

/// The PATH a new sign-in would get (user + machine from the registry), plus the current one:
/// tools winget just installed aren't in this process's PATH yet.
pub fn fresh_path_dirs() -> Vec<PathBuf> {
	let mut dirs = Vec::new();
	let expand = |s: String| -> String {
		let mut out = s.clone();
		for (k, v) in std::env::vars() {
			let pat = format!("%{k}%");
			if out.to_lowercase().contains(&pat.to_lowercase()) {
				let lower = out.to_lowercase();
				let at = lower.find(&pat.to_lowercase()).unwrap();
				out.replace_range(at..at + pat.len(), &v);
			}
		}
		out
	};
	let mut push = |s: String| {
		for d in s.split(';') {
			let d = d.trim();
			if !d.is_empty() {
				dirs.push(PathBuf::from(expand(d.to_string())));
			}
		}
	};
	if let Ok(k) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Environment") {
		if let Ok(v) = k.get_value::<String, _>("Path") {
			push(v);
		}
	}
	if let Ok(k) =
		RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")
	{
		if let Ok(v) = k.get_value::<String, _>("Path") {
			push(v);
		}
	}
	if let Some(p) = std::env::var_os("PATH") {
		push(p.to_string_lossy().to_string());
	}
	dirs
}

pub fn find_on_path(exe_name: &str) -> Option<PathBuf> {
	fresh_path_dirs().into_iter().map(|d| d.join(exe_name)).find(|p| p.is_file())
}

/// Starts a GUI program on its own, not tied to this process.
pub fn spawn_detached(exe: &Path, args: &[&str], cwd: &Path) -> std::io::Result<u32> {
	let child = Command::new(exe)
		.args(args)
		.current_dir(cwd)
		.stdin(Stdio::null())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
		.spawn()?;
	Ok(child.id())
}

/// Starts `cmd.exe /c <line>` with the line passed verbatim (cmd doesn't follow the C runtime's
/// quoting rules that std's argument escaping produces).
pub fn spawn_cmd_raw(line: &str) -> std::io::Result<()> {
	let system = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
	Command::new(system.join(r"System32\cmd.exe"))
		.raw_arg(format!("/d /c \"{line}\""))
		.stdin(Stdio::null())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
		.spawn()
		.map(|_| ())
}

pub fn shell_open(target: &str) -> bool {
	let t = HSTRING::from(target);
	let r = unsafe { ShellExecuteW(None, &HSTRING::from("open"), &t, PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL) };
	r.0 as isize > 32
}

// ---------------------------------------------------------------------------------------------
// Registry (HKCU unless noted)

pub fn hkcu() -> RegKey {
	RegKey::predef(HKEY_CURRENT_USER)
}

pub fn get_dword(path: &str, name: &str) -> Option<u32> {
	hkcu().open_subkey_with_flags(path, KEY_READ).ok()?.get_value::<u32, _>(name).ok()
}

pub fn get_string(path: &str, name: &str) -> Option<String> {
	hkcu().open_subkey_with_flags(path, KEY_READ).ok()?.get_value::<String, _>(name).ok()
}

pub fn set_dword(path: &str, name: &str, value: u32) -> std::io::Result<()> {
	let (k, _) = hkcu().create_subkey(path)?;
	k.set_value(name, &value)
}

pub fn set_string(path: &str, name: &str, value: &str) -> std::io::Result<()> {
	let (k, _) = hkcu().create_subkey(path)?;
	k.set_value(name, &value)
}

/// Deletes a value; missing is success.
pub fn delete_value(path: &str, name: &str) -> std::io::Result<()> {
	match hkcu().open_subkey_with_flags(path, KEY_WRITE) {
		Ok(k) => match k.delete_value(name) {
			Ok(()) => Ok(()),
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
			Err(e) => Err(e),
		},
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
		Err(e) => Err(e),
	}
}

/// Puts a DWORD back to what it was: the old value, or no value at all.
pub fn restore_dword(path: &str, name: &str, value: Option<u32>) -> std::io::Result<()> {
	match value {
		Some(v) => set_dword(path, name, v),
		None => delete_value(path, name),
	}
}

pub fn delete_tree(path: &str) -> std::io::Result<()> {
	match hkcu().delete_subkey_all(path) {
		Ok(()) => Ok(()),
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
		Err(e) => Err(e),
	}
}

pub fn broadcast_setting_change(area: &str) {
	let w = wide(area);
	let mut result = 0usize;
	unsafe {
		SendMessageTimeoutW(
			HWND_BROADCAST,
			WM_SETTINGCHANGE,
			WPARAM(0),
			LPARAM(w.as_ptr() as isize),
			SMTO_ABORTIFHUNG,
			3000,
			Some(&mut result),
		)
	};
}

pub fn broadcast_font_change() {
	let mut result = 0usize;
	unsafe {
		SendMessageTimeoutW(
			HWND_BROADCAST,
			WM_FONTCHANGE,
			WPARAM(0),
			LPARAM(0),
			SMTO_ABORTIFHUNG,
			3000,
			Some(&mut result),
		)
	};
}

// ---------------------------------------------------------------------------------------------
// Shortcuts, Apps & features

pub fn create_shortcut(
	lnk: &Path,
	target: &Path,
	args: &str,
	workdir: &Path,
	icon: &Path,
	description: &str,
) -> windows::core::Result<()> {
	if let Some(dir) = lnk.parent() {
		let _ = std::fs::create_dir_all(dir);
	}
	unsafe {
		let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
		link.SetPath(&HSTRING::from(target.as_os_str()))?;
		link.SetArguments(&HSTRING::from(args))?;
		link.SetWorkingDirectory(&HSTRING::from(workdir.as_os_str()))?;
		link.SetIconLocation(&HSTRING::from(icon.as_os_str()), 0)?;
		link.SetDescription(&HSTRING::from(description))?;
		let file: IPersistFile = link.cast()?;
		file.Save(&HSTRING::from(lnk.as_os_str()), true)?;
	}
	Ok(())
}

pub struct UninstallEntry<'a> {
	pub display_name: &'a str,
	pub version: &'a str,
	pub publisher: &'a str,
	pub install_dir: &'a Path,
	pub setup_exe: &'a Path,
	pub size_kb: u32,
	pub install_date: &'a str,
	pub url: &'a str,
}

pub fn uninstall_key_path() -> String {
	format!(r"{UNINSTALL_ROOT}\{}", crate::UNINSTALL_KEY_NAME)
}

pub fn write_uninstall_entry(e: &UninstallEntry) -> std::io::Result<String> {
	let path = uninstall_key_path();
	let (k, _) = hkcu().create_subkey(&path)?;
	let exe = e.setup_exe.display().to_string();
	k.set_value("DisplayName", &e.display_name)?;
	k.set_value("DisplayVersion", &e.version)?;
	k.set_value("Publisher", &e.publisher)?;
	k.set_value("InstallLocation", &e.install_dir.display().to_string())?;
	k.set_value("DisplayIcon", &format!("{exe},0"))?;
	k.set_value("EstimatedSize", &e.size_kb)?;
	k.set_value("UninstallString", &format!("\"{exe}\" --uninstall"))?;
	k.set_value("ModifyPath", &format!("\"{exe}\""))?;
	k.set_value("NoRepair", &1u32)?;
	k.set_value("InstallDate", &e.install_date)?;
	k.set_value("URLInfoAbout", &e.url)?;
	k.set_value("HelpLink", &e.url)?;
	k.set_value("URLUpdateInfo", &format!("{}/releases", e.url))?;
	Ok(path)
}

// ---------------------------------------------------------------------------------------------
// Taskbar

const ABS_AUTOHIDE: u32 = 0x1;
const ABS_ALWAYSONTOP: u32 = 0x2;

pub fn taskbar_autohide() -> Option<bool> {
	let mut data = APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, ..Default::default() };
	let state = unsafe { SHAppBarMessage(ABM_GETSTATE, &mut data) } as u32;
	Some(state & ABS_AUTOHIDE != 0)
}

pub fn set_taskbar_autohide(on: bool) {
	let mut data = APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, ..Default::default() };
	let state = unsafe { SHAppBarMessage(ABM_GETSTATE, &mut data) } as u32;
	data.hWnd = unsafe { FindWindowW(&HSTRING::from("Shell_TrayWnd"), PCWSTR::null()) }.unwrap_or_default();
	data.lParam = LPARAM(((state & ABS_ALWAYSONTOP) | if on { ABS_AUTOHIDE } else { 0 }) as isize);
	unsafe { SHAppBarMessage(ABM_SETSTATE, &mut data) };
}

/// Shows every taskbar window again. ii hides them in its hover-only mode and shows them on
/// exit; this covers an ii that was stopped without getting the chance.
pub fn show_taskbars() {
	unsafe extern "system" fn each(hwnd: HWND, _: LPARAM) -> windows::core::BOOL {
		let mut cls = [0u16; 64];
		let n = unsafe { GetClassNameW(hwnd, &mut cls) } as usize;
		let name = String::from_utf16_lossy(&cls[..n]);
		if name == "Shell_TrayWnd" || name == "Shell_SecondaryTrayWnd" {
			let _ = unsafe { ShowWindow(hwnd, SW_SHOWNA) };
		}
		true.into()
	}
	let _ = unsafe { EnumWindows(Some(each), LPARAM(0)) };
}

// ---------------------------------------------------------------------------------------------
// Theme and wallpaper

pub fn read_dword_hkcu(path: &str, name: &str) -> Option<u32> {
	get_dword(path, name)
}

fn image_ext(path: &Path) -> &'static str {
	let mut head = [0u8; 8];
	if let Ok(mut f) = std::fs::File::open(path) {
		let _ = f.read(&mut head);
	}
	match head {
		[0xFF, 0xD8, 0xFF, ..] => "jpg",
		[0x89, b'P', b'N', b'G', ..] => "png",
		[b'B', b'M', ..] => "bmp",
		[b'G', b'I', b'F', ..] => "gif",
		_ => match path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref() {
			Some("png") => "png",
			Some("bmp") => "bmp",
			Some("gif") => "gif",
			Some("webp") => "webp",
			Some("jxr") => "jxr",
			_ => "jpg",
		},
	}
}

/// The wallpaper as Windows shows it now, with a copy of each picture in `backup_dir`
/// (relative names in the result). Copies over 64 MB are skipped.
pub fn record_wallpaper(backup_dir: &Path, install_dir: &Path) -> Result<WallpaperState, String> {
	let mut state = WallpaperState {
		background_type: get_dword(WALLPAPERS_KEY, "BackgroundType"),
		registry_path: get_string(DESKTOP_KEY, "WallPaper"),
		..Default::default()
	};
	let wp: IDesktopWallpaper = unsafe { CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL) }
		.map_err(|e| format!("IDesktopWallpaper: {e}"))?;
	unsafe {
		state.position = wp.GetPosition().ok().map(|p| p.0);
		state.background_color = wp.GetBackgroundColor().ok().map(|c| c.0);
		state.slideshow = wp.GetStatus().map(|s| s.0 & 0x2 != 0).unwrap_or(false);
		let count = wp.GetMonitorDevicePathCount().unwrap_or(0);
		let mut copies: Vec<(String, String)> = Vec::new();
		for i in 0..count {
			let Ok(id) = wp.GetMonitorDevicePathAt(i) else { continue };
			let monitor = pwstr_to_string(id);
			let path = wp.GetWallpaper(&HSTRING::from(monitor.as_str())).map(pwstr_to_string).unwrap_or_default();
			let mut backup = None;
			if !path.is_empty() {
				if let Some((_, b)) = copies.iter().find(|(p, _)| p.eq_ignore_ascii_case(&path)) {
					backup = Some(b.clone());
				} else {
					let src = PathBuf::from(&path);
					let small = std::fs::metadata(&src).map(|m| m.len() <= 64 << 20).unwrap_or(false);
					if src.is_file() && small {
						let _ = std::fs::create_dir_all(backup_dir);
						let name = format!("wallpaper-{}.{}", copies.len(), image_ext(&src));
						if std::fs::copy(&src, backup_dir.join(&name)).is_ok() {
							let rel = backup_dir
								.join(&name)
								.strip_prefix(install_dir)
								.map(|p| p.display().to_string())
								.unwrap_or(name);
							copies.push((path.clone(), rel.clone()));
							backup = Some(rel);
						}
					}
				}
			}
			state.monitors.push(MonitorWallpaper { monitor, path, backup });
		}
	}
	Ok(state)
}

fn in_themes_cache(path: &str, roaming: &Path) -> bool {
	path_in(Path::new(path), &roaming.join(r"Microsoft\Windows\Themes"))
}

/// Puts the recorded wallpaper back. Pictures still at their original place are used from
/// there; others from the install-time copy (via `scratch`, since Windows only keeps a
/// transcoded cache). Returns notes about what couldn't be restored exactly.
pub fn restore_wallpaper(
	state: &WallpaperState,
	install_dir: &Path,
	roaming: &Path,
	scratch: &Path,
) -> Result<Vec<String>, String> {
	let mut notes = Vec::new();
	let wp: IDesktopWallpaper = unsafe { CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL) }
		.map_err(|e| format!("IDesktopWallpaper: {e}"))?;
	let solid = state.background_type == Some(1) || state.monitors.iter().all(|m| m.path.is_empty());
	unsafe {
		if let Some(c) = state.background_color {
			let _ = wp.SetBackgroundColor(COLORREF(c));
		}
		if solid {
			wp.Enable(false).map_err(|e| format!("solid color background: {e}"))?;
			return Ok(notes);
		}
		if let Some(p) = state.position {
			let _ = wp.SetPosition(DESKTOP_WALLPAPER_POSITION(p));
		}
		let current: Vec<String> = (0..wp.GetMonitorDevicePathCount().unwrap_or(0))
			.filter_map(|i| wp.GetMonitorDevicePathAt(i).ok().map(pwstr_to_string))
			.collect();
		let mut failures = 0;
		for (i, m) in state.monitors.iter().enumerate() {
			if m.path.is_empty() {
				continue;
			}
			let original = PathBuf::from(&m.path);
			let source = if original.is_file() && !in_themes_cache(&m.path, roaming) {
				original
			} else if let Some(b) = &m.backup {
				let from = install_dir.join(b);
				let _ = std::fs::create_dir_all(scratch);
				let to =
					scratch.join(format!("restore-{i}-{}", from.file_name().unwrap_or_default().to_string_lossy()));
				if std::fs::copy(&from, &to).is_err() {
					failures += 1;
					continue;
				}
				to
			} else {
				failures += 1;
				continue;
			};
			// A monitor that's gone since install: nothing to put back there.
			let target = current.iter().find(|c| c.eq_ignore_ascii_case(&m.monitor)).cloned();
			let monitor = match (&target, current.len()) {
				(Some(t), _) => HSTRING::from(t.as_str()),
				(None, 1) if state.monitors.len() == 1 => HSTRING::from(current[0].as_str()),
				_ => continue,
			};
			if wp.SetWallpaper(&monitor, &HSTRING::from(source.as_os_str())).is_err() {
				failures += 1;
			}
		}
		if failures > 0 {
			notes.push(format!("{failures} wallpaper picture(s) could not be put back"));
		}
	}
	// Windows now points WallPaper at our temporary copy; when the original setting was its own
	// cache file (which now holds the restored picture again), point it back there.
	if let Some(reg) = &state.registry_path {
		if in_themes_cache(reg, roaming)
			&& state.monitors.iter().all(|m| m.path.eq_ignore_ascii_case(reg) || m.path.is_empty())
		{
			let _ = set_string(DESKTOP_KEY, "WallPaper", reg);
		}
	}
	if state.slideshow || matches!(state.background_type, Some(2) | Some(3)) {
		notes.push("slideshow/spotlight".into());
	}
	Ok(notes)
}

// ---------------------------------------------------------------------------------------------
// Fonts

/// Is a font with this file name, or this registry name, already installed for the user or the
/// machine?
pub fn font_present(paths: &Paths, file_name: &str, registry_name: &str) -> bool {
	if paths.user_fonts.join(file_name).exists() {
		return true;
	}
	let windir = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
	if windir.join("Fonts").join(file_name).exists() {
		return true;
	}
	let lower = file_name.to_ascii_lowercase();
	for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
		let Ok(k) = RegKey::predef(root).open_subkey_with_flags(FONTS_KEY, KEY_READ) else { continue };
		for (name, value) in k.enum_values().flatten() {
			if name.eq_ignore_ascii_case(registry_name) {
				return true;
			}
			if let Ok(data) = String::from_reg_value(&value) {
				let d = data.to_ascii_lowercase();
				if d == lower || d.ends_with(&format!("\\{lower}")) {
					return true;
				}
			}
		}
	}
	false
}

/// Registers a per-user font: HKCU Fonts value (full path, which is what makes it load at
/// every sign-in) plus AddFontResource for the current session.
pub fn register_font(file: &Path, registry_name: &str) -> std::io::Result<()> {
	set_string(FONTS_KEY, registry_name, &file.display().to_string())?;
	let w = wide(file.as_os_str());
	unsafe { AddFontResourceW(PCWSTR(w.as_ptr())) };
	Ok(())
}

pub fn unregister_font(file: &Path, registry_name: &str) -> std::io::Result<()> {
	let w = wide(file.as_os_str());
	// AddFontResource counts; take off every reference this session holds.
	for _ in 0..8 {
		if !unsafe { RemoveFontResourceW(PCWSTR(w.as_ptr())) }.as_bool() {
			break;
		}
	}
	delete_value(FONTS_KEY, registry_name)
}

// ---------------------------------------------------------------------------------------------
// PowerShell

pub fn powershell_exe() -> PathBuf {
	let system = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
	system.join(r"System32\WindowsPowerShell\v1.0\powershell.exe")
}

pub fn pwsh_exe() -> Option<PathBuf> {
	if let Some(p) = find_on_path("pwsh.exe") {
		return Some(p);
	}
	let pf = std::env::var_os("ProgramFiles").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Program Files"));
	let p = pf.join(r"PowerShell\7\pwsh.exe");
	p.is_file().then_some(p)
}

/// Runs a PowerShell snippet (passed as -EncodedCommand, so no quoting issues) and returns its
/// output, read as UTF-8.
pub fn ps(exe: &Path, script: &str, timeout: Duration) -> std::io::Result<Output> {
	let full = format!(
		"$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false); {script}"
	);
	let utf16: Vec<u8> = full.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
	let b64 = base64(&utf16);
	run(exe, &["-NoLogo", "-NoProfile", "-NonInteractive", "-EncodedCommand", &b64], timeout)
}

fn base64(data: &[u8]) -> String {
	const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
	let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
	for chunk in data.chunks(3) {
		let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
		let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
		out.push(T[(n >> 18) as usize & 63] as char);
		out.push(T[(n >> 12) as usize & 63] as char);
		out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
		out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
	}
	out
}

/// `$PROFILE` (CurrentUserCurrentHost) as that shell sees it: OneDrive can move Documents.
pub fn profile_path(exe: &Path) -> Result<PathBuf, String> {
	let out = ps(exe, "$PROFILE.CurrentUserCurrentHost", Duration::from_secs(60)).map_err(|e| e.to_string())?;
	let line = out.stdout.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or_default().to_string();
	if !out.success() || line.is_empty() {
		return Err(format!("could not read $PROFILE from {}: {}", exe.display(), out.text()));
	}
	Ok(PathBuf::from(line))
}

/// Windows PowerShell 5.1's execution policy for one scope ("Undefined" if unset), or with
/// `scope` = None the one a newly started Windows PowerShell gets: the first defined of the
/// Group Policy, CurrentUser and LocalMachine scopes, else Restricted (the client default).
/// The Process scope is left out: it belongs to whoever started this setup.
pub fn exec_policy(scope: Option<&str>) -> Option<String> {
	let script = "Get-ExecutionPolicy -List | ForEach-Object { '{0}={1}' -f $_.Scope, $_.ExecutionPolicy }";
	let out = ps(&powershell_exe(), script, Duration::from_secs(60)).ok()?;
	if !out.success() {
		return None;
	}
	let list: Vec<(String, String)> = out
		.stdout
		.lines()
		.filter_map(|l| l.trim().split_once('=').map(|(a, b)| (a.to_string(), b.to_string())))
		.collect();
	let get = |name: &str| list.iter().find(|(s, _)| s.eq_ignore_ascii_case(name)).map(|(_, v)| v.clone());
	match scope {
		Some(s) => get(s),
		None => {
			for s in ["MachinePolicy", "UserPolicy", "CurrentUser", "LocalMachine"] {
				if let Some(v) = get(s).filter(|v| !v.eq_ignore_ascii_case("Undefined")) {
					return Some(v);
				}
			}
			Some("Restricted".into())
		}
	}
}

pub fn set_exec_policy_current_user(value: &str) -> Result<(), String> {
	let allowed = ["Undefined", "Restricted", "AllSigned", "RemoteSigned", "Unrestricted", "Bypass", "Default"];
	if !allowed.iter().any(|a| a.eq_ignore_ascii_case(value)) {
		return Err(format!("unexpected execution policy {value}"));
	}
	// A Group Policy setting wins over CurrentUser and makes the cmdlet throw even though it
	// stored the value; reading it back below is what decides success.
	let script = format!(
		"try {{ Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy {value} -Force }} catch {{ Write-Output \"note: $($_.Exception.Message)\" }}"
	);
	let out = ps(&powershell_exe(), &script, Duration::from_secs(60)).map_err(|e| e.to_string())?;
	let now = exec_policy(Some("CurrentUser")).unwrap_or_default();
	if now.eq_ignore_ascii_case(value) {
		Ok(())
	} else {
		Err(format!("CurrentUser execution policy is {now} after setting {value}: {}", out.text()))
	}
}

/// Policies under which Windows PowerShell runs a local, unsigned profile script.
pub fn policy_allows_profiles(policy: &str) -> bool {
	["RemoteSigned", "Unrestricted", "Bypass"].iter().any(|p| p.eq_ignore_ascii_case(policy))
}

// ---------------------------------------------------------------------------------------------
// winget

pub fn winget_exe() -> Option<PathBuf> {
	let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from)?;
	let alias = local.join(r"Microsoft\WindowsApps\winget.exe");
	if alias.exists() {
		return Some(alias);
	}
	find_on_path("winget.exe")
}

pub fn winget_version(exe: &Path) -> Option<String> {
	let out = run(exe, &["--version"], Duration::from_secs(60)).ok()?;
	out.success().then(|| out.stdout.trim().to_string())
}

/// `Some(true)` if winget lists the package as installed (any scope), `None` if winget itself
/// failed in a way that says nothing about the package.
pub fn winget_installed(exe: &Path, id: &str) -> Option<bool> {
	let out = run(
		exe,
		&["list", "--id", id, "--exact", "--accept-source-agreements", "--disable-interactivity"],
		Duration::from_secs(180),
	)
	.ok()?;
	if out.timed_out {
		return None;
	}
	if out.success() {
		return Some(out.stdout.to_lowercase().contains(&id.to_lowercase()));
	}
	// APPINSTALLER_CLI_ERROR_NO_APPLICATIONS_FOUND
	match out.code {
		Some(c) if c as u32 == 0x8A15_0014 => Some(false),
		_ => None,
	}
}

pub fn winget_install(exe: &Path, id: &str, user_scope: bool) -> Result<Output, Output> {
	let mut args = vec!["install", "--id", id, "--exact", "--source", "winget", "--silent"];
	if user_scope {
		args.extend(["--scope", "user"]);
	}
	args.extend(["--accept-package-agreements", "--accept-source-agreements", "--disable-interactivity"]);
	let out = run(exe, &args, Duration::from_secs(900)).map_err(|e| Output {
		code: None,
		stdout: String::new(),
		stderr: e.to_string(),
		timed_out: false,
	})?;
	// APPINSTALLER_CLI_ERROR_PACKAGE_ALREADY_INSTALLED counts as done.
	if out.success() || out.code.map(|c| c as u32) == Some(0x8A15_002B) {
		Ok(out)
	} else {
		Err(out)
	}
}

pub fn winget_uninstall(exe: &Path, id: &str) -> Result<Output, Output> {
	let args =
		["uninstall", "--id", id, "--exact", "--silent", "--accept-source-agreements", "--disable-interactivity"];
	let out = run(exe, &args, Duration::from_secs(900)).map_err(|e| Output {
		code: None,
		stdout: String::new(),
		stderr: e.to_string(),
		timed_out: false,
	})?;
	if out.success() || out.code.map(|c| c as u32) == Some(0x8A15_0014) {
		Ok(out)
	} else {
		Err(out)
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn b64() {
		assert_eq!(super::base64(b"Man"), "TWFu");
		assert_eq!(super::base64(b"Ma"), "TWE=");
		assert_eq!(super::base64(b"M"), "TQ==");
	}
}

// ---------------------------------------------------------------------------------------------
// Single instance, message box

/// Holds a named mutex for the life of the process; `None` if another setup already holds it.
pub fn single_instance(name: &str) -> Option<windows::Win32::Foundation::HANDLE> {
	use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
	use windows::Win32::System::Threading::CreateMutexW;
	let h = unsafe { CreateMutexW(None, true, &HSTRING::from(name)) }.ok()?;
	if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
		let _ = unsafe { CloseHandle(h) };
		return None;
	}
	Some(h)
}

pub fn message_box(title: &str, text: &str) {
	use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};
	unsafe { MessageBoxW(None, &HSTRING::from(text), &HSTRING::from(title), MB_OK | MB_ICONINFORMATION) };
}
