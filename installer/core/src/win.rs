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
	DesktopWallpaper, FOLDERID_Desktop, FOLDERID_Documents, FOLDERID_LocalAppData, FOLDERID_Pictures,
	FOLDERID_Programs, FOLDERID_RoamingAppData, FOLDERID_Videos, IDesktopWallpaper, IShellLinkW, SHAppBarMessage,
	SHGetKnownFolderPath, ShellExecuteW, ShellLink, ABM_GETSTATE, ABM_SETSTATE, APPBARDATA,
	DESKTOP_WALLPAPER_POSITION, KF_FLAG_DEFAULT,
};
use windows::Win32::UI::WindowsAndMessaging::{
	EnumWindows, FindWindowW, GetClassNameW, SendMessageTimeoutW, ShowWindow, HWND_BROADCAST, SMTO_ABORTIFHUNG,
	SW_SHOWNA, SW_SHOWNORMAL, WM_FONTCHANGE, WM_SETTINGCHANGE,
};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use winreg::types::FromRegValue;
use winreg::RegKey;

use crate::knownfolders::KnownFolder;
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

pub fn known_folder(id: &GUID) -> Option<PathBuf> {
	let p = unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }.ok()?;
	let s = pwstr_to_string(p);
	(!s.is_empty()).then(|| PathBuf::from(s))
}

pub fn known_folder_id(folder: KnownFolder) -> &'static GUID {
	match folder {
		KnownFolder::Documents => &FOLDERID_Documents,
		KnownFolder::Desktop => &FOLDERID_Desktop,
		KnownFolder::Pictures => &FOLDERID_Pictures,
		KnownFolder::Videos => &FOLDERID_Videos,
	}
}

pub fn probe_folder_access(path: &Path) -> std::io::Result<()> {
	std::fs::File::open(path)?;
	if let Some(entry) = std::fs::read_dir(path)?.next() {
		entry?;
	}
	Ok(())
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
	lang & 0x3ff == 0x16
}

pub fn free_space(path: &Path) -> Option<u64> {
	let mut p = path.to_path_buf();
	while !p.exists() {
		p = p.parent()?.to_path_buf();
	}
	let w = wide(p.as_os_str());
	let mut free = 0u64;
	unsafe { GetDiskFreeSpaceExW(PCWSTR(w.as_ptr()), Some(&mut free), None, None) }.ok()?;
	Some(free)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Proc {
	pub pid: u32,
	pub name: String,
	pub path: PathBuf,
}

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

pub fn wait_exit(pid: u32, timeout: Duration) -> bool {
	let Ok(h) = (unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) }) else { return true };
	let r = unsafe { WaitForSingleObject(h, timeout.as_millis() as u32) };
	let _ = unsafe { CloseHandle(h) };
	r.0 == 0
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

pub fn run(exe: &Path, args: &[&str], timeout: Duration) -> std::io::Result<Output> {
	let mut child = Command::new(exe)
		.args(args)
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

pub fn windows_terminal_present() -> bool {
	find_on_path("wt.exe").is_some()
}

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

pub fn delete_generic_credential(target: &str) -> std::io::Result<bool> {
	use windows::Win32::Foundation::ERROR_NOT_FOUND;
	use windows::Win32::Security::Credentials::{CredDeleteW, CRED_TYPE_GENERIC};

	let name = wide(target);
	match unsafe { CredDeleteW(PCWSTR(name.as_ptr()), CRED_TYPE_GENERIC, None) } {
		Ok(()) => Ok(true),
		Err(e) if e.code() == ERROR_NOT_FOUND.to_hresult() => Ok(false),
		Err(e) => Err(std::io::Error::other(e.message())),
	}
}

pub fn centered_in_work_area(hwnd: isize, width: i32, height: i32) -> Option<(i32, i32)> {
	use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};

	let monitor = unsafe { MonitorFromWindow(HWND(hwnd as *mut _), MONITOR_DEFAULTTONEAREST) };
	let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
	if !unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
		return None;
	}
	let work = info.rcWork;
	let x = work.left + ((work.right - work.left - width) / 2).max(0);
	let y = work.top + ((work.bottom - work.top - height) / 2).max(0);
	Some((x, y))
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

pub fn register_font(file: &Path, registry_name: &str) -> std::io::Result<()> {
	set_string(FONTS_KEY, registry_name, &file.display().to_string())?;
	let w = wide(file.as_os_str());
	unsafe { AddFontResourceW(PCWSTR(w.as_ptr())) };
	Ok(())
}

pub fn unregister_font(file: &Path, registry_name: &str) -> std::io::Result<()> {
	let w = wide(file.as_os_str());
	for _ in 0..8 {
		if !unsafe { RemoveFontResourceW(PCWSTR(w.as_ptr())) }.as_bool() {
			break;
		}
	}
	delete_value(FONTS_KEY, registry_name)
}

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

pub fn profile_path(exe: &Path) -> Result<PathBuf, String> {
	let out = ps(exe, "$PROFILE.CurrentUserCurrentHost", Duration::from_secs(60)).map_err(|e| e.to_string())?;
	let line = out.stdout.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or_default().to_string();
	if !out.success() || line.is_empty() {
		return Err(format!("could not read $PROFILE from {}: {}", exe.display(), out.text()));
	}
	Ok(PathBuf::from(line))
}

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

pub fn policy_allows_profiles(policy: &str) -> bool {
	["RemoteSigned", "Unrestricted", "Bypass"].iter().any(|p| p.eq_ignore_ascii_case(policy))
}

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

fn ps_quote(s: &str) -> String {
	format!("'{}'", s.replace('\'', "''"))
}

pub fn install_appx_bundle(msixbundle: &Path, dependency_paths: &[PathBuf]) -> std::io::Result<Output> {
	let deps = dependency_paths.iter().map(|p| ps_quote(&p.display().to_string())).collect::<Vec<_>>().join(",");
	let script =
		format!("Add-AppxPackage -Path {} -DependencyPath @({deps})", ps_quote(&msixbundle.display().to_string()));
	ps(&powershell_exe(), &script, Duration::from_secs(600))
}

pub fn remove_appx_package(name: &str) -> std::io::Result<Output> {
	let script = format!("Get-AppxPackage -Name {} | Remove-AppxPackage", ps_quote(name));
	ps(&powershell_exe(), &script, Duration::from_secs(120))
}

#[cfg(test)]
mod tests {
	#[test]
	fn b64() {
		assert_eq!(super::base64(b"Man"), "TWFu");
		assert_eq!(super::base64(b"Ma"), "TWE=");
		assert_eq!(super::base64(b"M"), "TQ==");
	}

	#[test]
	fn ps_quoting() {
		assert_eq!(super::ps_quote(r"C:\Users\test\file.exe"), r"'C:\Users\test\file.exe'");
		assert_eq!(super::ps_quote("C:\\Users\\O'Brien\\file.exe"), "'C:\\Users\\O''Brien\\file.exe'");
	}
}

const WEBVIEW2_CLIENT_KEY: &str = r"Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

pub fn webview2_present() -> bool {
	let candidates = [
		(HKEY_LOCAL_MACHINE, format!(r"SOFTWARE\WOW6432Node\{WEBVIEW2_CLIENT_KEY}")),
		(HKEY_LOCAL_MACHINE, format!(r"SOFTWARE\{WEBVIEW2_CLIENT_KEY}")),
		(HKEY_CURRENT_USER, format!(r"Software\{WEBVIEW2_CLIENT_KEY}")),
	];
	candidates.iter().any(|(root, path)| {
		RegKey::predef(*root)
			.open_subkey_with_flags(path, KEY_READ)
			.ok()
			.and_then(|k| k.get_value::<String, _>("pv").ok())
			.is_some_and(|v| {
				let v = v.trim();
				!v.is_empty() && v != "0.0.0.0"
			})
	})
}

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

pub fn message_box_yes_no(title: &str, text: &str) -> bool {
	use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, IDYES, MB_ICONQUESTION, MB_YESNO};
	(unsafe { MessageBoxW(None, &HSTRING::from(text), &HSTRING::from(title), MB_YESNO | MB_ICONQUESTION) }) == IDYES
}
