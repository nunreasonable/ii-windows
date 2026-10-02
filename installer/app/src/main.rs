// The ii-windows setup: a Tauri window over iiw_setup_core. Windows only.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use iiw_setup_core::log::Log;
use iiw_setup_core::manifest::{Manifest, Options};
use iiw_setup_core::ops::{self, Action, Ctx, Preflight, RunOptions, Source};
use iiw_setup_core::paths::Paths;
use iiw_setup_core::progress::{Event, Msg, Reporter};
use iiw_setup_core::{package, readme, win, SETUP_EXE};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};

const SETUP_VERSION: &str = env!("IIW_SETUP_VERSION");
const README_EN: &str = include_str!("../../README.en.md");
const README_PT: &str = include_str!("../../README.pt-BR.md");
const EVENT: &str = "setup-event";

#[derive(Debug, Default, Clone)]
struct Args {
	page: Option<String>,
	package: Option<PathBuf>,
	relaunched: bool,
	session: Option<PathBuf>,
	origin: Option<PathBuf>,
}

fn parse_args() -> Args {
	let mut a = Args::default();
	let mut it = std::env::args_os().skip(1);
	while let Some(arg) = it.next() {
		match arg.to_string_lossy().to_ascii_lowercase().as_str() {
			"--install" => a.page = Some("install".into()),
			"--update" => a.page = Some("update".into()),
			"--repair" => a.page = Some("repair".into()),
			"--uninstall" => a.page = Some("uninstall".into()),
			"--package" => {
				a.package = it.next().map(|p| {
					let p = PathBuf::from(p);
					if p.is_absolute() {
						p
					} else {
						std::env::current_dir().unwrap_or_default().join(p)
					}
				})
			}
			"--relaunched" => a.relaunched = true,
			"--session" => a.session = it.next().map(PathBuf::from),
			"--origin" => a.origin = it.next().map(PathBuf::from),
			_ => {}
		}
	}
	a
}

fn new_session_dir(paths: &Paths) -> PathBuf {
	let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos();
	paths.temp.join(format!("ii-windows-setup-{}-{:x}", std::process::id(), nanos))
}

/// The copy in the install dir can't replace or delete its own folder while it runs, so it
/// starts a copy of itself from the temp folder and exits.
fn relaunch_from_temp(exe: &Path, paths: &Paths) -> bool {
	let session = new_session_dir(paths);
	if std::fs::create_dir_all(&session).is_err() {
		return false;
	}
	let copy = session.join(SETUP_EXE);
	if std::fs::copy(exe, &copy).is_err() {
		return false;
	}
	let mut cmd = std::process::Command::new(&copy);
	cmd.args(std::env::args_os().skip(1));
	cmd.arg("--relaunched").arg("--session").arg(&session);
	if let Some(dir) = exe.parent() {
		cmd.arg("--origin").arg(dir);
	}
	cmd.spawn().is_ok()
}

struct AppState {
	args: Args,
	paths: Paths,
	session: PathBuf,
	self_exe: PathBuf,
	offline: Option<PathBuf>,
	offline_error: Option<String>,
	running: Arc<AtomicBool>,
	cancel: Arc<AtomicBool>,
	sources: Mutex<Vec<(Action, Source)>>,
	lang_pt: bool,
}

#[derive(Serialize)]
struct InstalledInfo {
	version: String,
	installed_at: String,
	updated_at: Option<String>,
	repaired_at: Option<String>,
	options: Options,
	tools: Vec<String>,
	pwsh7_ours: bool,
	fonts: usize,
	profiles: Vec<String>,
	exec_policy_changed: bool,
	exec_policy_previous: Option<String>,
	has_pre_install: bool,
	state: String,
}

#[derive(Serialize)]
struct OfflineInfo {
	path: String,
	version: String,
	size: u64,
}

#[derive(Serialize)]
struct Info {
	setup_version: String,
	lang: String,
	readme_en: String,
	readme_pt: String,
	page: Option<String>,
	installed: Option<InstalledInfo>,
	manifest_error: Option<String>,
	offline: Option<OfflineInfo>,
	offline_error: Option<String>,
	install_dir: String,
	settings_dir: String,
	settings_exist: bool,
	unmanaged_files: bool,
}

#[tauri::command]
fn info(state: tauri::State<'_, AppState>) -> Info {
	let p = &state.paths;
	let (manifest, manifest_error) = match Manifest::load(&p.manifest()) {
		Ok(m) => (m, None),
		Err(e) => (None, Some(e.to_string())),
	};
	let installed = manifest.map(|m| InstalledInfo {
		tools: m.items.winget.iter().map(|w| w.id.clone()).filter(|id| id != iiw_setup_core::PWSH_WINGET_ID).collect(),
		pwsh7_ours: m.has_winget(iiw_setup_core::PWSH_WINGET_ID),
		fonts: m.items.fonts.len(),
		profiles: m.items.profiles.iter().map(|p| p.path.display().to_string()).collect(),
		exec_policy_changed: m.items.exec_policy.as_ref().is_some_and(|c| c.changed),
		exec_policy_previous: m.items.exec_policy.as_ref().map(|c| c.previous.clone()),
		has_pre_install: m.pre_install.is_some(),
		version: m.version,
		installed_at: m.installed_at,
		updated_at: m.updated_at,
		repaired_at: m.repaired_at,
		options: m.options,
		state: m.state,
	});
	let offline = state.offline.as_ref().and_then(|path| {
		package::inspect(path).ok().map(|i| OfflineInfo {
			path: path.display().to_string(),
			version: i.version,
			size: i.file_size,
		})
	});
	Info {
		setup_version: SETUP_VERSION.into(),
		lang: if state.lang_pt { "pt".into() } else { "en".into() },
		readme_en: readme::to_html(README_EN),
		readme_pt: readme::to_html(README_PT),
		page: state.args.page.clone(),
		unmanaged_files: installed.is_none() && p.qsw_exe().exists(),
		installed,
		manifest_error,
		offline,
		offline_error: state.offline_error.clone(),
		install_dir: p.install_dir.display().to_string(),
		settings_dir: p.settings.display().to_string(),
		settings_exist: p.settings.exists(),
	}
}

#[tauri::command]
async fn preflight(
	state: tauri::State<'_, AppState>,
	action: Action,
	terminal: bool,
	needed_mb: u64,
) -> Result<Preflight, String> {
	let paths = state.paths.clone();
	tauri::async_runtime::spawn_blocking(move || {
		let _com = win::Com::init();
		ops::preflight(&paths, action, needed_mb.max(1) << 20, terminal)
	})
	.await
	.map_err(|e| e.to_string())
}

/// Finds the package for an action (offline, latest release, or the installed version's
/// release for a repair) and remembers it for `start`.
#[tauri::command]
async fn source(state: tauri::State<'_, AppState>, action: Action) -> Result<Source, Msg> {
	let offline = state.offline.clone();
	let want = match action {
		Action::Repair => Manifest::load(&state.paths.manifest()).ok().flatten().map(|m| m.version),
		_ => None,
	};
	let src = tauri::async_runtime::spawn_blocking(move || ops::resolve_source(offline.as_deref(), want.as_deref()))
		.await
		.map_err(|e| Msg::new("internal", e.to_string()))??;
	let mut cache = state.sources.lock().unwrap();
	cache.retain(|(a, _)| *a != action);
	cache.push((action, src.clone()));
	Ok(src)
}

struct UiReporter {
	app: AppHandle,
	cancel: Arc<AtomicBool>,
}

impl Reporter for UiReporter {
	fn emit(&self, event: Event) {
		let _ = self.app.emit(EVENT, &event);
	}
	fn cancelled(&self) -> bool {
		self.cancel.load(Ordering::SeqCst)
	}
}

#[tauri::command]
fn start(app: AppHandle, state: tauri::State<'_, AppState>, action: Action, opts: RunOptions) -> Result<(), String> {
	if state.running.swap(true, Ordering::SeqCst) {
		return Err("already running".into());
	}
	let source = state.sources.lock().unwrap().iter().find(|(a, _)| *a == action).map(|(_, s)| s.clone());
	if action != Action::Uninstall && source.is_none() {
		state.running.store(false, Ordering::SeqCst);
		return Err("no package source resolved".into());
	}
	state.cancel.store(false, Ordering::SeqCst);
	let paths = state.paths.clone();
	let scratch = state.session.clone();
	let self_exe = state.self_exe.clone();
	let running = state.running.clone();
	let cancel = state.cancel.clone();
	std::thread::spawn(move || {
		let _com = win::Com::init();
		let reporter = UiReporter { app: app.clone(), cancel };
		let log_app = app.clone();
		let log = Log::new(Box::new(move |line| {
			let _ = log_app.emit(EVENT, &Event::Log { line: line.to_string() });
		}));
		let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
			let mut ctx = Ctx::new(paths, &reporter, &log, scratch, self_exe, SETUP_VERSION);
			match (action, source) {
				(Action::Install, Some(s)) => ops::install(&mut ctx, &s, &opts),
				(Action::Update, Some(s)) => ops::update(&mut ctx, &s, &opts),
				(Action::Repair, Some(s)) => ops::repair(&mut ctx, &s, &opts),
				(Action::Uninstall, _) => ops::uninstall(&mut ctx, &opts),
				_ => false,
			}
		}));
		if let Err(panic) = result {
			let text = panic
				.downcast_ref::<String>()
				.cloned()
				.or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
				.unwrap_or_else(|| "unknown".into());
			log.line(&format!("INTERNAL ERROR: {text}"));
			reporter.emit(Event::Finished {
				ok: false,
				error: Some(Msg::new("internal", format!("Internal error: {text}")).with("error", text)),
				warnings: Vec::new(),
				notes: Vec::new(),
				can_launch: false,
			});
		}
		running.store(false, Ordering::SeqCst);
	});
	Ok(())
}

#[tauri::command]
fn cancel(state: tauri::State<'_, AppState>) {
	state.cancel.store(true, Ordering::SeqCst);
}

#[tauri::command]
fn launch(state: tauri::State<'_, AppState>) -> Result<u32, String> {
	ops::launch_ii(&state.paths).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_url(url: String) -> bool {
	url.starts_with("https://") && win::shell_open(&url)
}

/// Opens a file or folder the setup told the user about (logs, backups): only under the
/// user's local app data or temp folder.
#[tauri::command]
fn open_path(state: tauri::State<'_, AppState>, path: String) -> bool {
	let p = PathBuf::from(&path);
	let allowed = win::path_in(&p, &state.paths.local) || win::path_in(&p, &state.paths.temp);
	if !allowed || !p.exists() {
		return false;
	}
	if p.is_dir() {
		win::shell_open(&path)
	} else {
		// Show it selected in Explorer rather than running whatever handles the extension.
		let _ = std::process::Command::new("explorer.exe").arg(format!("/select,{path}")).spawn();
		true
	}
}

#[tauri::command]
fn win_minimize(window: tauri::WebviewWindow) {
	let _ = window.minimize();
}

#[tauri::command]
fn win_close(window: tauri::WebviewWindow, state: tauri::State<'_, AppState>) -> bool {
	if state.running.load(Ordering::SeqCst) {
		return false;
	}
	let _ = window.close();
	true
}

#[tauri::command]
fn ready(window: tauri::WebviewWindow) {
	let _ = window.show();
	let _ = window.set_focus();
}

fn main() {
	let mut args = parse_args();
	let paths = win::detect_paths();
	let self_exe = std::env::current_exe().unwrap_or_default();

	if !args.relaunched && win::path_in(&self_exe, &paths.install_dir) && relaunch_from_temp(&self_exe, &paths) {
		return;
	}

	let _instance = match win::single_instance("Local\\ii-windows-setup") {
		Some(h) => h,
		None => {
			win::message_box("illogical-impulse Setup", "The setup is already running.");
			return;
		}
	};

	let session = args.session.clone().unwrap_or_else(|| new_session_dir(&paths));
	let _ = std::fs::create_dir_all(&session);

	// Offline source: --package, else a package next to the exe that was started.
	let origin = args.origin.clone().or_else(|| self_exe.parent().map(Path::to_path_buf));
	let mut offline_error = None;
	if let Some(p) = &args.package {
		if !p.is_file() {
			offline_error = Some(format!("--package: {} doesn't exist", p.display()));
			args.package = None;
		}
	}
	let offline = args.package.clone().or_else(|| origin.as_deref().and_then(package::find_offline));
	if let Some(p) = &offline {
		if let Err(e) = package::inspect(p) {
			offline_error = Some(format!("{}: {e}", p.display()));
		}
	}
	let offline = if offline_error.is_some() { None } else { offline };

	let state = AppState {
		args,
		paths,
		session: session.clone(),
		self_exe,
		offline,
		offline_error,
		running: Arc::new(AtomicBool::new(false)),
		cancel: Arc::new(AtomicBool::new(false)),
		sources: Mutex::new(Vec::new()),
		lang_pt: win::ui_language_is_portuguese(),
	};
	let webview_data = session.join("webview");

	let app = tauri::Builder::default()
		.manage(state)
		.invoke_handler(tauri::generate_handler![
			info,
			preflight,
			source,
			start,
			cancel,
			launch,
			open_url,
			open_path,
			win_minimize,
			win_close,
			ready
		])
		.setup(move |app| {
			WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
				.title("illogical-impulse Setup")
				.inner_size(980.0, 660.0)
				.resizable(false)
				.maximizable(false)
				.decorations(false)
				.shadow(true)
				.center()
				.visible(false)
				.background_color(tauri::window::Color(19, 19, 24, 255))
				.data_directory(webview_data.clone())
				.build()?;
			Ok(())
		})
		.on_window_event(|window, event| {
			if let WindowEvent::CloseRequested { api, .. } = event {
				let state = window.state::<AppState>();
				if state.running.load(Ordering::SeqCst) {
					api.prevent_close();
					let _ = window.emit("close-blocked", ());
				}
			}
		})
		.build(tauri::generate_context!())
		.expect("error while building the setup window");

	app.run(move |_handle, event| {
		if let RunEvent::Exit = event {
			ops::schedule_cleanup(&session, &[]);
		}
	});
}
