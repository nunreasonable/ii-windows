//! Install, update, repair and uninstall, step by step. Every change goes into the manifest as
//! soon as it's made, so an interrupted run can still be undone.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::fsops::{self, Swap};
use crate::log::{self, Log};
use crate::manifest::{ExecPolicyChange, Font, Manifest, Options, PreInstall, ProfileEdit, RunValue, WingetPackage};
use crate::package::{self, PackageInfo};
use crate::paths::Paths;
use crate::progress::{Event, Msg, Reporter, Status};
use crate::release::{self, ReleaseInfo};
use crate::{profile, ttf, version, win};

type R<T> = Result<T, Msg>;

/// Entries of the install dir that belong to the installer, not to the package.
const KEEP: [&str; 6] =
	[crate::MANIFEST, crate::LOG, "restore", "backup", crate::SETUP_EXE, "install-manifest.json.tmp"];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
	Install,
	Update,
	Repair,
	Uninstall,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RunOptions {
	pub options: Options,
	pub launch: bool,
	/// Uninstall: winget-uninstall the terminal tools this installer installed.
	pub remove_tools: bool,
	/// Uninstall: also PowerShell 7, if this installer installed it.
	pub remove_pwsh7: bool,
	pub keep_settings: bool,
	/// Uninstall: put back the wallpaper, light/dark mode, accent color and taskbar auto-hide.
	pub restore_look: bool,
	/// Update: reinstall even when the release isn't newer.
	pub force: bool,
}

/// Where the package comes from.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
	Offline { path: PathBuf, version: String, size: u64 },
	Github { release: ReleaseInfo },
}

impl Source {
	pub fn version(&self) -> &str {
		match self {
			Source::Offline { version, .. } => version,
			Source::Github { release } => &release.version,
		}
	}
	pub fn download_size(&self) -> u64 {
		match self {
			Source::Offline { .. } => 0,
			Source::Github { release } => release.zip_size,
		}
	}
	fn describe(&self) -> String {
		match self {
			Source::Offline { path, .. } => format!("offline package {}", path.display()),
			Source::Github { release } => format!("GitHub release {} ({})", release.tag, release.html_url),
		}
	}
}

pub fn msg_from_release_error(e: &release::Error) -> Msg {
	match e {
		release::Error::NotFound => Msg::new("no_release", e.to_string()),
		release::Error::Network(x) => Msg::new("network", e.to_string()).with("error", x.clone()),
		release::Error::NoPackage(t) => Msg::new("no_package_asset", e.to_string()).with("tag", t.clone()),
		release::Error::Integrity(x) => Msg::new("checksum_mismatch", x.clone()),
		release::Error::Io(x) => Msg::new("io", x.to_string()).with("error", x.to_string()),
		release::Error::Cancelled => Msg::new("cancelled", "cancelled"),
	}
}

/// The package to use: the offline one if given, else the latest GitHub release (or, for a
/// repair, the release of `want_version`).
pub fn resolve_source(offline: Option<&Path>, want_version: Option<&str>) -> R<Source> {
	if let Some(p) = offline {
		let info =
			package::inspect(p).map_err(|e| Msg::new("package_invalid", e.to_string()).with("error", e.to_string()))?;
		let use_it = want_version.is_none_or(|v| version::same(v, &info.version));
		if use_it {
			return Ok(Source::Offline { path: p.to_path_buf(), version: info.version, size: info.file_size });
		}
	}
	let rel = match want_version {
		Some(v) => release::for_version(v),
		None => release::latest(),
	};
	rel.map(|release| Source::Github { release }).map_err(|e| msg_from_release_error(&e))
}

// ---------------------------------------------------------------------------------------------

pub struct Ctx<'a> {
	pub paths: Paths,
	pub reporter: &'a dyn Reporter,
	pub log: &'a Log,
	/// This run's temp folder (downloads, wallpaper copies); deleted when the setup exits.
	pub scratch: PathBuf,
	/// The running setup exe (copied into the install dir).
	pub self_exe: PathBuf,
	pub setup_version: String,
	warnings: Vec<Msg>,
	notes: Vec<Msg>,
}

impl<'a> Ctx<'a> {
	pub fn new(
		paths: Paths,
		reporter: &'a dyn Reporter,
		log: &'a Log,
		scratch: PathBuf,
		self_exe: PathBuf,
		setup_version: &str,
	) -> Ctx<'a> {
		Ctx {
			paths,
			reporter,
			log,
			scratch,
			self_exe,
			setup_version: setup_version.into(),
			warnings: Vec::new(),
			notes: Vec::new(),
		}
	}

	fn info(&self, text: impl AsRef<str>) {
		self.log.line(text.as_ref());
	}

	fn plan(&self, steps: &[&str]) {
		self.reporter.emit(Event::Plan { steps: steps.iter().map(|s| s.to_string()).collect() });
	}

	fn begin(&self, id: &str) {
		self.info(format!("== {id}"));
		self.reporter.emit(Event::Step { id: id.into(), status: Status::Running, detail: None });
	}

	fn done(&self, id: &str) {
		self.reporter.emit(Event::Step { id: id.into(), status: Status::Done, detail: None });
	}

	fn done_with(&self, id: &str, detail: Msg) {
		self.info(format!("   {}", detail.text));
		self.reporter.emit(Event::Step { id: id.into(), status: Status::Done, detail: Some(detail) });
	}

	fn skip(&self, id: &str, detail: Msg) {
		self.info(format!("   skipped: {}", detail.text));
		self.reporter.emit(Event::Step { id: id.into(), status: Status::Skipped, detail: Some(detail) });
	}

	fn warn(&mut self, id: &str, detail: Msg) {
		self.info(format!("   WARNING: {}", detail.text));
		self.reporter.emit(Event::Step { id: id.into(), status: Status::Warning, detail: Some(detail.clone()) });
		self.warnings.push(detail);
	}

	fn fail(&self, id: &str, detail: &Msg) {
		self.info(format!("   FAILED: {}", detail.text));
		self.reporter.emit(Event::Step { id: id.into(), status: Status::Failed, detail: Some(detail.clone()) });
	}

	fn progress(&self, id: &str, fraction: f64, text: Option<String>) {
		self.reporter.emit(Event::Progress { id: id.into(), fraction, text });
	}

	fn note(&mut self, m: Msg) {
		self.info(format!("   note: {}", m.text));
		self.notes.push(m);
	}

	fn finish(&mut self, result: R<()>, can_launch: bool) -> bool {
		let ok = result.is_ok();
		if self.log.path().is_none() {
			// Nothing was installed (the run failed before the install dir existed): keep the
			// log in the temp folder instead.
			let p = self.paths.temp.join(format!("ii-windows-setup-{}.log", log::stamp()));
			self.log.open(&p);
			self.notes.push(
				Msg::new("setup_log", format!("The log of this run is {}", p.display()))
					.with("path", p.display().to_string()),
			);
		}
		match &result {
			Ok(()) => self.info(format!("finished OK ({} warning(s))", self.warnings.len())),
			Err(e) => self.info(format!("finished with an error: {}", e.text)),
		}
		self.reporter.emit(Event::Finished {
			ok,
			error: result.err(),
			warnings: std::mem::take(&mut self.warnings),
			notes: std::mem::take(&mut self.notes),
			can_launch: can_launch && ok,
		});
		ok
	}
}

fn io_msg(key: &str, what: impl std::fmt::Display, e: impl std::fmt::Display) -> Msg {
	Msg::new(key, format!("{what}: {e}")).with("error", e.to_string()).with("path", what.to_string())
}

fn mb(bytes: u64) -> String {
	if bytes >= 1 << 30 {
		format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
	} else {
		format!("{:.0} MB", bytes as f64 / 1_048_576.0)
	}
}

// ---------------------------------------------------------------------------------------------
// Preflight (read-only)

#[derive(Debug, Clone, Serialize)]
pub struct Check {
	pub id: String,
	/// "ok", "info", "warn" or "error" (error blocks the action).
	pub level: String,
	pub msg: Msg,
}

#[derive(Debug, Clone, Serialize)]
pub struct Preflight {
	pub checks: Vec<Check>,
	pub windows_build: u32,
	pub winget: bool,
	pub pwsh: Option<PathBuf>,
	pub exec_policy_current_user: Option<String>,
	pub exec_policy_effective: Option<String>,
	pub running_in_install_dir: Vec<win::Proc>,
	pub running_elsewhere: Vec<win::Proc>,
	pub free_bytes: Option<u64>,
	pub blocked: bool,
}

fn check(id: &str, level: &str, msg: Msg) -> Check {
	Check { id: id.into(), level: level.into(), msg }
}

pub fn ii_processes(paths: &Paths) -> (Vec<win::Proc>, Vec<win::Proc>) {
	let all = win::find_processes(&["qsw.exe", "qs.exe"]);
	all.into_iter().partition(|p| win::path_in(&p.path, &paths.install_dir))
}

/// What the options page shows before anything runs. `needed` is the disk space the action
/// needs on the %LOCALAPPDATA% volume.
pub fn preflight(paths: &Paths, action: Action, needed: u64, terminal: bool) -> Preflight {
	let mut checks = Vec::new();
	let (build, display) = win::windows_build();
	if build >= 22000 {
		checks.push(check(
			"windows",
			"ok",
			Msg::new("win_ok", format!("Windows 11 build {build} {display}"))
				.with("build", build.to_string())
				.with("display", display.clone()),
		));
	} else {
		checks.push(check(
			"windows",
			"warn",
			Msg::new(
				"win_old",
				format!("Windows build {build}: ii-windows is made for Windows 11 (build 22000 or newer)"),
			)
			.with("build", build.to_string()),
		));
	}

	let winget = win::winget_exe().is_some_and(|w| win::winget_version(&w).is_some());
	if matches!(action, Action::Install | Action::Repair) && terminal {
		if winget {
			checks.push(check("winget", "ok", Msg::new("winget_ok", "winget is available")));
		} else {
			checks.push(check(
				"winget",
				"warn",
				Msg::new("winget_missing", "winget (App Installer) was not found: the terminal tools will be skipped"),
			));
		}
	}

	let free = win::free_space(&paths.local);
	if matches!(action, Action::Install | Action::Update | Action::Repair) {
		match free {
			Some(f) if f < needed => checks.push(check(
				"disk",
				"error",
				Msg::new("disk_low", format!("Not enough disk space: {} needed, {} free", mb(needed), mb(f)))
					.with("need", mb(needed))
					.with("free", mb(f)),
			)),
			Some(f) => checks.push(check(
				"disk",
				"ok",
				Msg::new("disk_ok", format!("{} free ({} needed)", mb(f), mb(needed)))
					.with("need", mb(needed))
					.with("free", mb(f)),
			)),
			None => {}
		}
	}

	let manifest = Manifest::load(&paths.manifest()).ok().flatten();
	match (&manifest, action) {
		(Some(m), _) => checks.push(check(
			"existing",
			"info",
			Msg::new("installed_version", format!("ii-windows {} is installed", m.version))
				.with("version", m.version.clone()),
		)),
		(None, Action::Install) if paths.install_dir.join("qsw.exe").exists() => checks.push(check(
			"existing",
			"warn",
			Msg::new(
				"unmanaged_install",
				format!(
					"{} has files from an install this setup didn't make; they will be replaced",
					paths.install_dir.display()
				),
			)
			.with("path", paths.install_dir.display().to_string()),
		)),
		(None, Action::Uninstall | Action::Repair | Action::Update) => checks.push(check(
			"existing",
			"error",
			Msg::new("not_installed", "ii-windows isn't installed by this setup (no install manifest)"),
		)),
		_ => {}
	}

	let (mine, others) = ii_processes(paths);
	if !mine.is_empty() {
		checks.push(check(
			"running",
			"info",
			Msg::new("running_will_stop", "illogical-impulse is running and will be closed first"),
		));
	}
	for p in &others {
		checks.push(check(
			"running_elsewhere",
			"warn",
			Msg::new("running_elsewhere", format!("Another copy of Quickshell is running from {}. Close it first: while ii runs, the taskbar settings the installer records are ii's, not yours.", p.path.display()))
				.with("path", p.path.display().to_string()),
		));
	}

	let pwsh = win::pwsh_exe();
	let (cu, eff) = if matches!(action, Action::Install) {
		(win::exec_policy(Some("CurrentUser")), win::exec_policy(None))
	} else {
		(None, None)
	};
	let blocked = checks.iter().any(|c| c.level == "error");
	Preflight {
		checks,
		windows_build: build,
		winget,
		pwsh,
		exec_policy_current_user: cu,
		exec_policy_effective: eff,
		running_in_install_dir: mine,
		running_elsewhere: others,
		free_bytes: free,
		blocked,
	}
}

// ---------------------------------------------------------------------------------------------
// Shared steps

/// Downloads/verifies (or checks the offline package) and inspects it.
fn obtain_package(ctx: &mut Ctx, source: &Source) -> R<PackageInfo> {
	match source {
		Source::Offline { path, .. } => {
			ctx.begin("verify");
			let sha_path = PathBuf::from(format!("{}.sha256", path.display()));
			if sha_path.is_file() {
				let text = std::fs::read_to_string(&sha_path).map_err(|e| io_msg("io", sha_path.display(), e))?;
				let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
				let want = release::parse_sha256_file(&text, &name).map_err(|e| msg_from_release_error(&e))?;
				let got = release::sha256_file(path).map_err(|e| io_msg("io", path.display(), e))?;
				if got != want {
					let m = Msg::new(
						"checksum_mismatch",
						format!("SHA-256 of {} is {got}, expected {want}", path.display()),
					);
					ctx.fail("verify", &m);
					return Err(m);
				}
				ctx.done_with("verify", Msg::new("checksum_ok", format!("SHA-256 matches ({got})")).with("hash", got));
			} else {
				ctx.warn(
					"verify",
					Msg::new(
						"checksum_missing",
						format!("No {} next to the offline package: its integrity wasn't checked", sha_path.display()),
					),
				);
			}
			package::inspect(path).map_err(|e| {
				let m = Msg::new("package_invalid", e.to_string()).with("error", e.to_string());
				ctx.fail("verify", &m);
				m
			})
		}
		Source::Github { release: rel } => {
			ctx.begin("download");
			let free_tmp = win::free_space(&ctx.scratch).unwrap_or(u64::MAX);
			if free_tmp < rel.zip_size + (64 << 20) {
				let m = Msg::new("disk_low", format!("Not enough space in {} for the download", ctx.scratch.display()))
					.with("need", mb(rel.zip_size))
					.with("free", mb(free_tmp));
				ctx.fail("download", &m);
				return Err(m);
			}
			let sha_text = release::fetch_text(&rel.sha_url).map_err(|e| {
				let m = msg_from_release_error(&e);
				ctx.fail("download", &m);
				m
			})?;
			let want = release::parse_sha256_file(&sha_text, &rel.zip_name).map_err(|e| msg_from_release_error(&e))?;
			let dest = ctx.scratch.join("download").join(&rel.zip_name);
			ctx.info(format!("   {} -> {}", rel.zip_url, dest.display()));
			let mut last = 0.0;
			let reporter = ctx.reporter;
			let got = release::download(
				&rel.zip_url,
				&dest,
				rel.zip_size,
				&mut |done, total| {
					let f = if total > 0 { done as f64 / total as f64 } else { 0.0 };
					if f - last >= 0.005 || done == total {
						last = f;
						reporter.emit(Event::Progress {
							id: "download".into(),
							fraction: f,
							text: Some(format!("{} / {}", mb(done), mb(total))),
						});
					}
				},
				&|| reporter.cancelled(),
			)
			.map_err(|e| {
				let m = msg_from_release_error(&e);
				ctx.fail("download", &m);
				m
			})?;
			ctx.done_with(
				"download",
				Msg::new("downloaded", format!("{} ({})", rel.zip_name, mb(rel.zip_size)))
					.with("name", rel.zip_name.clone())
					.with("size", mb(rel.zip_size)),
			);
			ctx.begin("verify");
			if got != want {
				let m =
					Msg::new("checksum_mismatch", format!("SHA-256 of the download is {got}, the release says {want}"));
				ctx.fail("verify", &m);
				let _ = std::fs::remove_file(&dest);
				return Err(m);
			}
			let info = package::inspect(&dest).map_err(|e| {
				let m = Msg::new("package_invalid", e.to_string()).with("error", e.to_string());
				ctx.fail("verify", &m);
				m
			})?;
			ctx.done_with("verify", Msg::new("checksum_ok", format!("SHA-256 matches ({got})")).with("hash", got));
			Ok(info)
		}
	}
}

/// Closes ii if it runs from the install dir: politely through `qs kill` (so it restores the
/// taskbar on its way out), then forcibly if it doesn't exit. Nothing else is touched.
/// Returns whether anything was running.
fn stop_ii(ctx: &mut Ctx, manifest: Option<&Manifest>) -> bool {
	ctx.begin("stop");
	let (mine, _) = ii_processes(&ctx.paths);
	if mine.is_empty() {
		ctx.skip("stop", Msg::new("not_running", "illogical-impulse wasn't running"));
		return false;
	}
	let qs = ctx.paths.qs_exe();
	let mut forced = false;
	for p in &mine {
		ctx.info(format!("   stopping {} (pid {})", p.path.display(), p.pid));
		if qs.is_file() {
			let pid = p.pid.to_string();
			match win::run(&qs, &["kill", "--pid", &pid], Duration::from_secs(15)) {
				Ok(out) => ctx.info(format!("   qs kill --pid {pid}: exit {:?} {}", out.code, out.text())),
				Err(e) => ctx.info(format!("   qs kill failed: {e}")),
			}
		}
		if !win::wait_exit(p.pid, Duration::from_secs(10)) {
			ctx.info(format!("   pid {} didn't exit, terminating it", p.pid));
			win::terminate(p.pid);
			forced = true;
		}
	}
	if forced {
		// ii didn't get to undo its hover-only taskbar: show the taskbars and, if ii is the one
		// that turned auto-hide on, turn it back off.
		win::show_taskbars();
		if let Some(pre) = manifest.and_then(|m| m.pre_install.as_ref()) {
			if pre.taskbar_autohide == Some(false) && win::taskbar_autohide() == Some(true) {
				win::set_taskbar_autohide(false);
			}
		}
	}
	std::thread::sleep(Duration::from_millis(300));
	ctx.done_with(
		"stop",
		Msg::new("stopped", format!("closed {} process(es)", mine.len())).with("count", mine.len().to_string()),
	);
	true
}

fn record_pre_install(ctx: &mut Ctx) -> PreInstall {
	ctx.begin("record");
	let (_, others) = ii_processes(&ctx.paths);
	let mut pre = PreInstall {
		recorded_at: log::timestamp(),
		taskbar_autohide: win::taskbar_autohide(),
		apps_use_light_theme: win::get_dword(win::PERSONALIZE_KEY, "AppsUseLightTheme"),
		system_uses_light_theme: win::get_dword(win::PERSONALIZE_KEY, "SystemUsesLightTheme"),
		accent_color: win::get_dword(win::DWM_KEY, "AccentColor"),
		colorization_color: win::get_dword(win::DWM_KEY, "ColorizationColor"),
		colorization_afterglow: win::get_dword(win::DWM_KEY, "ColorizationAfterglow"),
		other_instance_running: !others.is_empty(),
		..Default::default()
	};
	match win::record_wallpaper(&ctx.paths.restore_dir(), &ctx.paths.install_dir) {
		Ok(w) => {
			ctx.info(format!("   wallpaper: {:?}", w));
			pre.wallpaper = Some(w);
		}
		Err(e) => ctx.warn(
			"record",
			Msg::new("wallpaper_record_failed", format!("Couldn't record the wallpaper: {e}")).with("error", e),
		),
	}
	ctx.info(format!(
		"   taskbar auto-hide {:?}, AppsUseLightTheme {:?}, SystemUsesLightTheme {:?}, AccentColor {:?}, ColorizationColor {:?}",
		pre.taskbar_autohide, pre.apps_use_light_theme, pre.system_uses_light_theme, pre.accent_color, pre.colorization_color
	));
	ctx.done("record");
	pre
}

/// Unpacks the package and swaps it into the install dir. The returned Swap is committed by
/// the caller once the config is in place too.
fn install_files(ctx: &mut Ctx, pkg: &PackageInfo) -> R<Swap> {
	ctx.begin("files");
	let staging = ctx.paths.staging();
	fsops::remove_any(&staging).map_err(|e| io_msg("files_failed", staging.display(), e))?;
	if let Some(free) = win::free_space(&ctx.paths.local) {
		let need = pkg.unpacked_size + (64 << 20);
		if free < need {
			let m = Msg::new("disk_low", format!("Not enough disk space: {} needed, {} free", mb(need), mb(free)))
				.with("need", mb(need))
				.with("free", mb(free));
			ctx.fail("files", &m);
			return Err(m);
		}
	}
	let reporter = ctx.reporter;
	let mut last = 0.0;
	package::extract(
		pkg,
		&staging,
		&mut |done, total| {
			let f = if total > 0 { done as f64 / total as f64 } else { 0.0 };
			if f - last >= 0.01 || done == total {
				last = f;
				reporter.emit(Event::Progress {
					id: "files".into(),
					fraction: f * 0.9,
					text: Some(format!("{} / {}", mb(done), mb(total))),
				});
			}
		},
		&|| reporter.cancelled(),
	)
	.map_err(|e| {
		let _ = fsops::remove_any(&staging);
		let m = if matches!(e, package::Error::Cancelled) {
			Msg::new("cancelled", "cancelled")
		} else {
			Msg::new("files_failed", e.to_string()).with("error", e.to_string())
		};
		ctx.fail("files", &m);
		m
	})?;
	// Dev-only folders a package might carry; never part of an install.
	let _ = fsops::remove_any(&staging.join("testconfigs"));
	for script in ["install.ps1", "uninstall.ps1"] {
		let _ = fsops::remove_any(&staging.join(script));
	}
	ctx.progress("files", 0.95, None);
	let swap = Swap::run(&staging, &ctx.paths.install_dir, &ctx.paths.previous(), &KEEP).map_err(|e| {
		let _ = fsops::remove_any(&staging);
		let m =
			Msg::new("files_in_use", format!("Couldn't replace the program files: {e}")).with("error", e.to_string());
		ctx.fail("files", &m);
		m
	})?;
	ctx.done_with(
		"files",
		Msg::new("files_ok", format!("{} files in {}", pkg.files, ctx.paths.install_dir.display()))
			.with("count", pkg.files.to_string())
			.with("path", ctx.paths.install_dir.display().to_string()),
	);
	Ok(swap)
}

/// The ii config (`%LOCALAPPDATA%\quickshell\ii`) replaced by the package's `config\ii`; the
/// old copy stays aside until the run commits.
struct ConfigSwap {
	old: Option<PathBuf>,
}

impl ConfigSwap {
	fn commit(self) {
		if let Some(old) = self.old {
			let _ = fsops::remove_any(&old);
		}
	}
}

fn install_config(ctx: &mut Ctx) -> R<ConfigSwap> {
	ctx.begin("config");
	let src = ctx.paths.install_dir.join("config").join("ii");
	let live = ctx.paths.ii_config.clone();
	let fresh = ctx.paths.quickshell.join("ii.new");
	let old = ctx.paths.quickshell.join("ii.old");
	let fail = |ctx: &Ctx, e: String| {
		let m = Msg::new("config_failed", format!("Couldn't install the ii config into {}: {e}", live.display()))
			.with("error", e)
			.with("path", live.display().to_string());
		ctx.fail("config", &m);
		m
	};
	let _ = fsops::remove_any(&fresh);
	let _ = fsops::remove_any(&old);
	fsops::copy_dir(&src, &fresh).map_err(|e| fail(ctx, e.to_string()))?;
	let had_old = live.exists();
	if had_old && fsops::rename_retry(&live, &old).is_err() {
		// Something holds the folder open (a terminal in it, another Quickshell watching it):
		// keep a copy for rollback and update it in place instead.
		ctx.info("   ii config folder is in use; updating it in place");
		fsops::copy_dir(&live, &old).map_err(|e| fail(ctx, e.to_string()))?;
		if let Err(e) = fsops::mirror_dir(&fresh, &live, &[]) {
			let _ = fsops::mirror_dir(&old, &live, &[]);
			let _ = fsops::remove_any(&old);
			let _ = fsops::remove_any(&fresh);
			return Err(fail(ctx, e.to_string()));
		}
		let _ = fsops::remove_any(&fresh);
		ctx.done("config");
		return Ok(ConfigSwap { old: Some(old) });
	}
	if let Err(e) = fsops::rename_retry(&fresh, &live) {
		if had_old {
			let _ = fsops::rename_retry(&old, &live);
		}
		let _ = fsops::remove_any(&fresh);
		return Err(fail(ctx, e.to_string()));
	}
	ctx.done_with(
		"config",
		Msg::new("config_ok", format!("ii config in {}", live.display())).with("path", live.display().to_string()),
	);
	Ok(ConfigSwap { old: had_old.then_some(old) })
}

fn seed_colors(ctx: &mut Ctx, m: &mut Manifest, force: bool) {
	let colors = ctx.paths.colors.clone();
	if colors.exists() && !force {
		ctx.info(format!("   {} exists, kept", colors.display()));
		return;
	}
	let src = ctx.paths.default_colors();
	let r = colors.parent().map(std::fs::create_dir_all).unwrap_or(Ok(())).and_then(|_| std::fs::copy(&src, &colors));
	match r {
		Ok(_) => {
			ctx.info(format!("   seeded {}", colors.display()));
			m.items.colors_seeded = true;
		}
		Err(e) => ctx.warn("config", io_msg("colors_failed", colors.display(), e)),
	}
}

fn copy_setup_exe(ctx: &mut Ctx, pkg: &PackageInfo) {
	let dest = ctx.paths.setup_exe();
	if pkg.has_setup_exe {
		ctx.info("   setup exe: the package's own copy");
		return;
	}
	if win::same_path(&ctx.self_exe, &dest) {
		return;
	}
	match std::fs::copy(&ctx.self_exe, &dest) {
		Ok(_) => ctx.info(format!("   copied the setup to {}", dest.display())),
		Err(e) => ctx.warn("shortcuts", io_msg("setup_copy_failed", dest.display(), e)),
	}
}

fn save(ctx: &mut Ctx, m: &Manifest) {
	if let Err(e) = m.save(&ctx.paths.manifest()) {
		ctx.warn("finish", io_msg("manifest_failed", ctx.paths.manifest().display(), e));
	}
}

fn shortcuts_and_entry(ctx: &mut Ctx, m: &mut Manifest) {
	ctx.begin("shortcuts");
	let p = ctx.paths.clone();
	let icon = p.setup_exe();
	let main = p.start_menu.join(crate::SHORTCUT_MAIN);
	let settings = p.start_menu.join(crate::SHORTCUT_SETTINGS);
	let settings_args = format!("-p \"{}\"", p.settings_qml().display());
	let mut ok = true;
	for (lnk, args, desc) in [
		(&main, "-c ii".to_string(), "illogical-impulse shell"),
		(&settings, settings_args, "illogical-impulse settings"),
	] {
		match win::create_shortcut(lnk, &p.qsw_exe(), &args, &p.install_dir, &icon, desc) {
			Ok(()) => {
				if !m.items.shortcuts.iter().any(|s| win::same_path(s, lnk)) {
					m.items.shortcuts.push(lnk.clone());
				}
			}
			Err(e) => {
				ok = false;
				ctx.warn(
					"shortcuts",
					Msg::new("shortcut_failed", format!("Couldn't create {}: {e}", lnk.display()))
						.with("path", lnk.display().to_string())
						.with("error", e.to_string()),
				);
			}
		}
	}
	let size_kb = (fsops::dir_size(&p.install_dir) / 1024).min(u32::MAX as u64) as u32;
	let entry = win::UninstallEntry {
		display_name: crate::DISPLAY_NAME,
		version: &m.version,
		publisher: crate::PUBLISHER,
		install_dir: &p.install_dir,
		setup_exe: &p.setup_exe(),
		size_kb,
		install_date: &log::date_yyyymmdd(),
		url: release::REPO_URL,
	};
	match win::write_uninstall_entry(&entry) {
		Ok(key) => m.items.uninstall_key = Some(key),
		Err(e) => {
			ok = false;
			ctx.warn(
				"shortcuts",
				Msg::new("arp_failed", format!("Couldn't register in Apps & features: {e}"))
					.with("error", e.to_string()),
			);
		}
	}
	if ok {
		ctx.done("shortcuts");
	}
}

fn run_command(p: &Paths) -> String {
	format!("\"{}\" -c ii", p.qsw_exe().display())
}

fn set_autostart(ctx: &mut Ctx, m: &mut Manifest, on: bool) {
	ctx.begin("autostart");
	let name = crate::RUN_VALUE;
	if on {
		let current = win::get_string(win::RUN_KEY, name);
		let wanted = run_command(&ctx.paths);
		if m.items.run_value.is_none() {
			// A value pointing into our own install dir is a leftover of an earlier install,
			// not something of the user's to put back later.
			let previous =
				current.filter(|v| !v.to_lowercase().contains(&ctx.paths.install_dir.to_string_lossy().to_lowercase()));
			m.items.run_value = Some(RunValue { name: name.into(), previous, set: false });
		}
		match win::set_string(win::RUN_KEY, name, &wanted) {
			Ok(()) => {
				if let Some(r) = m.items.run_value.as_mut() {
					r.set = true;
				}
				ctx.done("autostart");
			}
			Err(e) => ctx.warn("autostart", io_msg("autostart_failed", "HKCU Run", e)),
		}
	} else if let Some(r) = m.items.run_value.take() {
		let res = match &r.previous {
			Some(prev) => win::set_string(win::RUN_KEY, name, prev),
			None => win::delete_value(win::RUN_KEY, name),
		};
		match res {
			Ok(()) => ctx.done_with("autostart", Msg::new("autostart_off", "Start with Windows turned off")),
			Err(e) => {
				m.items.run_value = Some(r);
				ctx.warn("autostart", io_msg("autostart_failed", "HKCU Run", e));
			}
		}
	} else {
		ctx.skip("autostart", Msg::new("autostart_not_set", "not starting with Windows"));
	}
}

// --- Terminal ---------------------------------------------------------------------------------

fn package_fonts(p: &Paths) -> Vec<PathBuf> {
	let mut v: Vec<PathBuf> = std::fs::read_dir(p.install_dir.join("fonts"))
		.map(|rd| {
			rd.flatten()
				.map(|e| e.path())
				.filter(|f| {
					let n = f.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
					n.starts_with("jetbrainsmononerdfont-") && n.ends_with(".ttf")
				})
				.collect()
		})
		.unwrap_or_default();
	v.sort();
	v
}

fn install_fonts(ctx: &mut Ctx, m: &mut Manifest) {
	ctx.begin("fonts");
	let fonts = package_fonts(&ctx.paths);
	if fonts.is_empty() {
		ctx.warn("fonts", Msg::new("fonts_missing", "The package has no JetBrainsMono Nerd Font files"));
		return;
	}
	let (mut added, mut present) = (0, 0);
	for src in fonts {
		let file_name = src.file_name().unwrap_or_default().to_string_lossy().to_string();
		let dest = ctx.paths.user_fonts.join(&file_name);
		let data = match std::fs::read(&src) {
			Ok(d) => d,
			Err(e) => {
				ctx.warn("fonts", io_msg("font_failed", src.display(), e));
				continue;
			}
		};
		let reg = ttf::registry_value_name(&data, &file_name);
		let ours = m.items.fonts.iter().any(|f| win::same_path(&f.file, &dest));
		if !ours && win::font_present(&ctx.paths, &file_name, &reg) {
			present += 1;
			continue;
		}
		if ours && dest.is_file() {
			present += 1;
			continue;
		}
		let r = std::fs::create_dir_all(&ctx.paths.user_fonts)
			.and_then(|_| std::fs::write(&dest, &data))
			.and_then(|_| win::register_font(&dest, &reg));
		match r {
			Ok(()) => {
				added += 1;
				if !ours {
					m.items.fonts.push(Font { file: dest.clone(), registry_name: reg.clone() });
				}
				ctx.info(format!("   installed {file_name} as \"{reg}\""));
			}
			Err(e) => ctx.warn("fonts", io_msg("font_failed", dest.display(), e)),
		}
		save(ctx, m);
	}
	if added > 0 {
		win::broadcast_font_change();
	}
	ctx.done_with(
		"fonts",
		Msg::new("fonts_result", format!("{added} installed, {present} already present"))
			.with("added", added.to_string())
			.with("present", present.to_string()),
	);
}

fn remove_fonts(ctx: &mut Ctx, m: &mut Manifest, step: &str) -> Vec<PathBuf> {
	let mut leftover = Vec::new();
	let fonts = std::mem::take(&mut m.items.fonts);
	for f in &fonts {
		if let Err(e) = win::unregister_font(&f.file, &f.registry_name) {
			ctx.info(format!("   unregister {}: {e}", f.registry_name));
		}
	}
	if !fonts.is_empty() {
		win::broadcast_font_change();
	}
	for f in &fonts {
		match fsops::remove_any(&f.file) {
			Ok(()) => ctx.info(format!("   removed {}", f.file.display())),
			Err(e) => {
				ctx.info(format!(
					"   {} is in use ({e}); it will be deleted after this setup closes",
					f.file.display()
				));
				leftover.push(f.file.clone());
			}
		}
	}
	if fonts.is_empty() {
		ctx.skip(step, Msg::new("nothing_to_remove", "nothing installed by this setup"));
	}
	leftover
}

fn install_tools(ctx: &mut Ctx, m: &mut Manifest) {
	ctx.begin("tools");
	let Some(winget) = win::winget_exe() else {
		ctx.warn(
			"tools",
			Msg::new(
				"winget_missing",
				"winget (App Installer) was not found: Oh My Posh, Starship and eza were not installed",
			),
		);
		return;
	};
	let mut failed = false;
	let total = crate::TERMINAL_TOOLS.len();
	for (i, (id, cmd)) in crate::TERMINAL_TOOLS.iter().enumerate() {
		ctx.progress("tools", i as f64 / total as f64, Some((*id).to_string()));
		let listed = win::winget_installed(&winget, id);
		let on_path = win::find_on_path(&format!("{cmd}.exe"));
		if listed == Some(true) || on_path.is_some() {
			ctx.info(format!(
				"   {id}: already installed ({})",
				on_path.map(|p| p.display().to_string()).unwrap_or_else(|| "winget list".into())
			));
			continue;
		}
		ctx.info(format!("   winget install --scope user {id}"));
		match win::winget_install(&winget, id, true) {
			Ok(out) => {
				ctx.info(format!("   {}", out.text().replace('\n', "\n   ")));
				if !m.has_winget(id) {
					m.items.winget.push(WingetPackage { id: (*id).into(), installed_at: log::timestamp() });
				}
				save(ctx, m);
			}
			Err(out) => {
				failed = true;
				ctx.info(format!("   {}", out.text().replace('\n', "\n   ")));
				ctx.warn(
					"tools",
					Msg::new("tool_failed", format!("winget couldn't install {id} (exit {:?})", out.code))
						.with("id", (*id).to_string())
						.with("code", format!("{:?}", out.code.map(|c| c as u32))),
				);
			}
		}
	}
	ctx.progress("tools", 1.0, None);
	if !failed {
		let ours: Vec<String> =
			m.items.winget.iter().filter(|p| p.id != crate::PWSH_WINGET_ID).map(|p| p.id.clone()).collect();
		ctx.done_with(
			"tools",
			Msg::new(
				"tools_result",
				format!("installed by this setup: {}", if ours.is_empty() { "none".into() } else { ours.join(", ") }),
			)
			.with("list", ours.join(", ")),
		);
	}
}

fn install_pwsh7(ctx: &mut Ctx, m: &mut Manifest) {
	ctx.begin("pwsh7");
	if let Some(p) = win::pwsh_exe() {
		ctx.skip(
			"pwsh7",
			Msg::new("pwsh_present", format!("PowerShell 7 is already installed ({})", p.display()))
				.with("path", p.display().to_string()),
		);
		return;
	}
	let Some(winget) = win::winget_exe() else {
		ctx.warn(
			"pwsh7",
			Msg::new("winget_missing_pwsh", "winget (App Installer) was not found: PowerShell 7 was not installed"),
		);
		return;
	};
	ctx.info("   winget install Microsoft.PowerShell (asks for administrator permission)");
	match win::winget_install(&winget, crate::PWSH_WINGET_ID, false) {
		Ok(out) => {
			ctx.info(format!("   {}", out.text().replace('\n', "\n   ")));
			if !m.has_winget(crate::PWSH_WINGET_ID) {
				m.items.winget.push(WingetPackage { id: crate::PWSH_WINGET_ID.into(), installed_at: log::timestamp() });
			}
			save(ctx, m);
			ctx.done("pwsh7");
		}
		Err(out) => {
			ctx.info(format!("   {}", out.text().replace('\n', "\n   ")));
			ctx.warn(
				"pwsh7",
				Msg::new(
					"pwsh_failed",
					format!(
						"PowerShell 7 wasn't installed (winget exit {:?}; was the administrator prompt declined?)",
						out.code
					),
				),
			);
		}
	}
}

fn shells() -> Vec<(&'static str, PathBuf)> {
	let mut v = vec![("powershell", win::powershell_exe())];
	if let Some(p) = win::pwsh_exe() {
		v.push(("pwsh", p));
	}
	v
}

fn add_profile_blocks(ctx: &mut Ctx, m: &mut Manifest) {
	ctx.begin("profiles");
	let mut changed = Vec::new();
	let mut failed = false;
	for (shell, exe) in shells() {
		let path = match win::profile_path(&exe) {
			Ok(p) => p,
			Err(e) => {
				failed = true;
				ctx.warn("profiles", Msg::new("profile_failed", e.clone()).with("shell", shell).with("error", e));
				continue;
			}
		};
		let existed = path.is_file();
		let current = if existed { std::fs::read(&path) } else { Ok(Vec::new()) };
		let current = match current {
			Ok(c) => c,
			Err(e) => {
				failed = true;
				ctx.warn("profiles", io_msg("profile_failed", path.display(), e).with("shell", shell));
				continue;
			}
		};
		let record_idx = m.items.profiles.iter().position(|p| win::same_path(&p.path, &path));
		match profile::add_block(&current) {
			Ok(None) => {
				ctx.info(format!("   {shell}: {} already has the block", path.display()));
				if record_idx.is_none() {
					m.items.profiles.push(ProfileEdit {
						shell: shell.into(),
						path: path.clone(),
						..Default::default()
					});
				}
			}
			Ok(Some(new)) => {
				let mut backup = None;
				if existed {
					let b = ctx.paths.backup_dir().join("profiles").join(format!(
						"{}-{shell}-{}",
						log::stamp(),
						path.file_name().unwrap_or_default().to_string_lossy()
					));
					let r = b
						.parent()
						.map(std::fs::create_dir_all)
						.unwrap_or(Ok(()))
						.and_then(|_| std::fs::copy(&path, &b));
					if let Err(e) = r {
						failed = true;
						ctx.warn("profiles", io_msg("profile_failed", b.display(), e).with("shell", shell));
						continue;
					}
					backup = Some(b);
				}
				let dir = path.parent().map(Path::to_path_buf);
				let created_dir = dir.as_ref().filter(|d| !d.exists()).cloned();
				let r = dir.map(std::fs::create_dir_all).unwrap_or(Ok(())).and_then(|_| std::fs::write(&path, &new));
				match r {
					Ok(()) => {
						changed.push(shell);
						ctx.info(format!("   {shell}: added the block to {}", path.display()));
						match record_idx {
							Some(i) => {
								let rec = &mut m.items.profiles[i];
								rec.block_added = true;
								if !existed {
									rec.created_file = true;
								}
								if rec.created_dir.is_none() {
									rec.created_dir = created_dir;
								}
								if rec.backup.is_none() {
									rec.backup = backup;
								}
							}
							None => m.items.profiles.push(ProfileEdit {
								shell: shell.into(),
								path: path.clone(),
								created_file: !existed,
								created_dir,
								block_added: true,
								backup,
							}),
						}
						save(ctx, m);
					}
					Err(e) => {
						failed = true;
						ctx.warn("profiles", io_msg("profile_failed", path.display(), e).with("shell", shell));
					}
				}
			}
			Err(e) => {
				failed = true;
				ctx.warn(
					"profiles",
					Msg::new("profile_failed", format!("{}: {e}", path.display()))
						.with("shell", shell)
						.with("error", e.to_string()),
				);
			}
		}
	}
	if !failed {
		ctx.done_with(
			"profiles",
			Msg::new("profiles_result", format!("block present in {} profile(s)", m.items.profiles.len()))
				.with("count", m.items.profiles.len().to_string()),
		);
	}
}

/// Takes the marked block out of every profile the manifest knows. Returns backup copies made.
fn remove_profile_blocks(ctx: &mut Ctx, m: &mut Manifest, step: &str, backup_root: &Path) {
	let records = std::mem::take(&mut m.items.profiles);
	if records.is_empty() {
		ctx.skip(step, Msg::new("nothing_to_remove", "nothing installed by this setup"));
		return;
	}
	let mut kept = Vec::new();
	for rec in records {
		let path = rec.path.clone();
		let Ok(current) = std::fs::read(&path) else {
			ctx.info(format!("   {} is gone", path.display()));
			continue;
		};
		match profile::remove_block(&current) {
			Ok(None) => ctx.info(format!("   {}: no block", path.display())),
			Ok(Some(new)) => {
				let b = backup_root.join(format!(
					"{}-{}",
					rec.shell,
					path.file_name().unwrap_or_default().to_string_lossy()
				));
				let _ = std::fs::create_dir_all(backup_root);
				if let Err(e) = std::fs::copy(&path, &b) {
					ctx.warn(step, io_msg("profile_failed", b.display(), e).with("shell", rec.shell.clone()));
					kept.push(rec);
					continue;
				}
				let res = if rec.created_file && profile::is_blank(&new) {
					fsops::remove_any(&path).map(|_| {
						if let Some(d) = &rec.created_dir {
							let _ = std::fs::remove_dir(d); // only if empty
						}
					})
				} else {
					std::fs::write(&path, &new)
				};
				match res {
					Ok(()) => {
						ctx.info(format!("   removed the block from {} (copy before: {})", path.display(), b.display()))
					}
					Err(e) => {
						ctx.warn(step, io_msg("profile_failed", path.display(), e).with("shell", rec.shell.clone()));
						kept.push(rec);
					}
				}
			}
			Err(e) => {
				ctx.warn(
					step,
					Msg::new("profile_failed", format!("{}: {e}", path.display()))
						.with("shell", rec.shell.clone())
						.with("error", e.to_string()),
				);
				kept.push(rec);
			}
		}
	}
	m.items.profiles = kept;
}

fn apply_exec_policy(ctx: &mut Ctx, m: &mut Manifest, on: bool) {
	ctx.begin("exec_policy");
	if on {
		if m.items.exec_policy.as_ref().is_some_and(|c| c.changed) {
			ctx.skip("exec_policy", Msg::new("policy_already_set", "already set by this setup"));
			return;
		}
		let current = win::exec_policy(Some("CurrentUser")).unwrap_or_else(|| "Undefined".into());
		let effective = win::exec_policy(None).unwrap_or_default();
		if win::policy_allows_profiles(&effective) {
			ctx.skip(
				"exec_policy",
				Msg::new("policy_allows", format!("Windows PowerShell already allows profile scripts ({effective})"))
					.with("policy", effective),
			);
			return;
		}
		match win::set_exec_policy_current_user("RemoteSigned") {
			Ok(()) => {
				m.items.exec_policy = Some(ExecPolicyChange { changed: true, previous: current.clone() });
				save(ctx, m);
				let eff = win::exec_policy(None).unwrap_or_default();
				ctx.done_with(
					"exec_policy",
					Msg::new("policy_set", format!("CurrentUser: {current} -> RemoteSigned (effective: {eff})"))
						.with("previous", current)
						.with("effective", eff),
				);
			}
			Err(e) => ctx.warn("exec_policy", Msg::new("policy_failed", e.clone()).with("error", e)),
		}
	} else if let Some(change) = m.items.exec_policy.take().filter(|c| c.changed) {
		restore_exec_policy(ctx, &change, "exec_policy");
		if win::exec_policy(Some("CurrentUser")).is_some_and(|v| !v.eq_ignore_ascii_case(&change.previous)) {
			m.items.exec_policy = Some(change);
		}
	} else {
		ctx.skip("exec_policy", Msg::new("policy_untouched", "left as it is"));
	}
}

fn restore_exec_policy(ctx: &mut Ctx, change: &ExecPolicyChange, step: &str) {
	let prev = if change.previous.is_empty() { "Undefined" } else { &change.previous };
	match win::set_exec_policy_current_user(prev) {
		Ok(()) => ctx.done_with(
			step,
			Msg::new("policy_restored", format!("CurrentUser execution policy back to {prev}")).with("policy", prev),
		),
		Err(e) => ctx.warn(step, Msg::new("policy_failed", e.clone()).with("error", e)),
	}
}

fn launch(ctx: &mut Ctx) {
	ctx.begin("launch");
	match win::spawn_detached(&ctx.paths.qsw_exe(), &["-c", "ii"], &ctx.paths.install_dir) {
		Ok(pid) => ctx.done_with("launch", Msg::new("launched", format!("started (pid {pid})"))),
		Err(e) => ctx.warn("launch", io_msg("launch_failed", ctx.paths.qsw_exe().display(), e)),
	}
}

pub fn launch_ii(paths: &Paths) -> std::io::Result<u32> {
	win::spawn_detached(&paths.qsw_exe(), &["-c", "ii"], &paths.install_dir)
}

/// Program files + config, with rollback if the config part fails. On a fresh install that
/// fails, everything the run created goes away again.
fn swap_in(ctx: &mut Ctx, pkg: &PackageInfo, fresh: bool) -> R<()> {
	let mut swap = match install_files(ctx, pkg) {
		Ok(s) => s,
		Err(e) => {
			if fresh {
				let _ = fsops::remove_any(&ctx.paths.install_dir);
			}
			return Err(e);
		}
	};
	match install_config(ctx) {
		Ok(cfg) => {
			if let Err(e) = swap.commit() {
				ctx.info(format!("   couldn't delete {}: {e}", ctx.paths.previous().display()));
			}
			cfg.commit();
			Ok(())
		}
		Err(e) => {
			for p in swap.rollback() {
				ctx.info(format!("   rollback problem: {p}"));
			}
			if fresh {
				let _ = fsops::remove_any(&ctx.paths.install_dir);
			}
			Err(e)
		}
	}
}

fn apply_integrations(ctx: &mut Ctx, m: &mut Manifest, opts: &Options, previous: &Options) {
	shortcuts_and_entry(ctx, m);
	save(ctx, m);
	set_autostart(ctx, m, opts.autostart);
	save(ctx, m);
	if opts.terminal {
		install_fonts(ctx, m);
		install_tools(ctx, m);
	} else if previous.terminal {
		ctx.begin("fonts");
		let had = !m.items.fonts.is_empty();
		let left = remove_fonts(ctx, m, "fonts");
		schedule_leftovers(ctx, &left);
		if had {
			ctx.done("fonts");
		}
	}
	if opts.pwsh7 {
		install_pwsh7(ctx, m);
	}
	if opts.terminal {
		add_profile_blocks(ctx, m);
	} else if previous.terminal {
		ctx.begin("profiles");
		let had = !m.items.profiles.is_empty();
		let root = ctx.paths.backup_dir().join("profiles").join(log::stamp());
		remove_profile_blocks(ctx, m, "profiles", &root);
		if had {
			ctx.done("profiles");
		}
	}
	save(ctx, m);
	if opts.exec_policy || m.items.exec_policy.as_ref().is_some_and(|c| c.changed) {
		apply_exec_policy(ctx, m, opts.exec_policy);
	}
	m.options = opts.clone();
	save(ctx, m);
}

fn integration_steps(opts: &Options, previous: &Options, had_policy: bool) -> Vec<&'static str> {
	let mut s = vec!["shortcuts", "autostart"];
	if opts.terminal || previous.terminal {
		s.push("fonts");
	}
	if opts.terminal {
		s.push("tools");
	}
	if opts.pwsh7 {
		s.push("pwsh7");
	}
	if opts.terminal || previous.terminal {
		s.push("profiles");
	}
	if opts.exec_policy || had_policy {
		s.push("exec_policy");
	}
	s
}

fn source_steps(source: &Source) -> Vec<&'static str> {
	match source {
		Source::Offline { .. } => vec!["verify"],
		Source::Github { .. } => vec!["download", "verify"],
	}
}

// ---------------------------------------------------------------------------------------------
// The four actions

pub fn install(ctx: &mut Ctx, source: &Source, run: &RunOptions) -> bool {
	let existing = Manifest::load(&ctx.paths.manifest()).ok().flatten();
	let fresh = existing.is_none();
	let previous_opts = existing.as_ref().map(|m| m.options.clone()).unwrap_or_default();
	let had_policy = existing.as_ref().and_then(|m| m.items.exec_policy.as_ref()).is_some_and(|c| c.changed);
	let opts = run.options.clone();

	let mut plan = source_steps(source);
	plan.push("stop");
	if existing.as_ref().is_none_or(|m| m.pre_install.is_none()) {
		plan.push("record");
	}
	plan.extend(["files", "config"]);
	plan.extend(integration_steps(&opts, &previous_opts, had_policy));
	plan.push("finish");
	if run.launch {
		plan.push("launch");
	}
	ctx.plan(&plan);
	ctx.info(format!("ii-windows setup {} - install from {}", ctx.setup_version, source.describe()));
	ctx.info(format!("options: {opts:?}, previous: {previous_opts:?}"));

	let result = (|| -> R<()> {
		let pkg = obtain_package(ctx, source)?;
		stop_ii(ctx, existing.as_ref());
		let mut m = existing.clone().unwrap_or_else(|| Manifest::new(&ctx.paths.install_dir));
		if m.pre_install.is_none() {
			m.pre_install = Some(record_pre_install(ctx));
		}
		swap_in(ctx, &pkg, fresh)?;
		ctx.log.open(&ctx.paths.log());
		m.version = pkg.version.clone();
		m.setup_version = ctx.setup_version.clone();
		m.source = source.describe();
		m.install_dir = ctx.paths.install_dir.clone();
		if fresh {
			m.installed_at = log::timestamp();
		} else {
			m.updated_at = Some(log::timestamp());
		}
		m.state = "installing".into();
		seed_colors(ctx, &mut m, false);
		copy_setup_exe(ctx, &pkg);
		save(ctx, &m);
		apply_integrations(ctx, &mut m, &opts, &previous_opts);
		ctx.begin("finish");
		m.state = "installed".into();
		save(ctx, &m);
		ctx.done("finish");
		if opts.terminal {
			ctx.note(Msg::new("restart_terminal", "Open a new terminal window to see the new prompt and colors"));
		}
		if run.launch {
			launch(ctx);
		}
		Ok(())
	})();
	ctx.finish(result, !run.launch)
}

pub fn update(ctx: &mut Ctx, source: &Source, run: &RunOptions) -> bool {
	let Some(mut m) = Manifest::load(&ctx.paths.manifest()).ok().flatten() else {
		return ctx.finish(Err(Msg::new("not_installed", "ii-windows isn't installed by this setup")), false);
	};
	let mut plan = source_steps(source);
	plan.extend(["stop", "files", "config", "shortcuts", "finish"]);
	plan.push("launch");
	ctx.plan(&plan);
	ctx.log.open(&ctx.paths.log());
	ctx.info(format!(
		"ii-windows setup {} - update {} -> {} from {}",
		ctx.setup_version,
		m.version,
		source.version(),
		source.describe()
	));
	let result = (|| -> R<()> {
		if !run.force && !version::is_newer(source.version(), &m.version) {
			return Err(Msg::new("up_to_date", format!("{} is already the latest version", m.version))
				.with("version", m.version.clone()));
		}
		let pkg = obtain_package(ctx, source)?;
		let was_running = stop_ii(ctx, Some(&m));
		ctx.log.close();
		let r = swap_in(ctx, &pkg, false);
		ctx.log.open(&ctx.paths.log());
		r?;
		let from = m.version.clone();
		m.version = pkg.version.clone();
		m.setup_version = ctx.setup_version.clone();
		m.source = source.describe();
		m.updated_at = Some(log::timestamp());
		seed_colors(ctx, &mut m, false);
		copy_setup_exe(ctx, &pkg);
		shortcuts_and_entry(ctx, &mut m);
		if m.options.autostart {
			let _ = win::set_string(win::RUN_KEY, crate::RUN_VALUE, &run_command(&ctx.paths));
		}
		ctx.begin("finish");
		m.state = "installed".into();
		save(ctx, &m);
		ctx.done_with(
			"finish",
			Msg::new("updated", format!("{from} -> {}", m.version)).with("from", from).with("to", m.version.clone()),
		);
		if was_running || run.launch {
			launch(ctx);
		} else {
			ctx.skip("launch", Msg::new("not_running", "ii wasn't running"));
		}
		Ok(())
	})();
	ctx.finish(result, false)
}

pub fn repair(ctx: &mut Ctx, source: &Source, run: &RunOptions) -> bool {
	let Some(mut m) = Manifest::load(&ctx.paths.manifest()).ok().flatten() else {
		return ctx.finish(Err(Msg::new("not_installed", "ii-windows isn't installed by this setup")), false);
	};
	let opts = m.options.clone();
	let had_policy = m.items.exec_policy.as_ref().is_some_and(|c| c.changed);
	let mut plan = source_steps(source);
	plan.extend(["stop", "settings", "files", "config"]);
	plan.extend(integration_steps(&opts, &opts, had_policy));
	plan.extend(["finish", "launch"]);
	ctx.plan(&plan);
	ctx.log.open(&ctx.paths.log());
	ctx.info(format!("ii-windows setup {} - repair {} from {}", ctx.setup_version, m.version, source.describe()));
	let result = (|| -> R<()> {
		let pkg = obtain_package(ctx, source)?;
		stop_ii(ctx, Some(&m));

		ctx.begin("settings");
		let backup = ctx.paths.settings.join("backups").join(format!("repair-{}", log::stamp()));
		let mut moved = 0;
		if let Ok(rd) = std::fs::read_dir(&ctx.paths.settings) {
			for e in rd.flatten() {
				let p = e.path();
				let is_json = p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("json"));
				if !is_json {
					continue;
				}
				let r = std::fs::create_dir_all(&backup)
					.and_then(|_| std::fs::copy(&p, backup.join(e.file_name())))
					.and_then(|_| std::fs::remove_file(&p));
				match r {
					Ok(()) => moved += 1,
					Err(err) => {
						let m = io_msg("settings_failed", p.display(), err);
						ctx.fail("settings", &m);
						return Err(m);
					}
				}
			}
		}
		if moved > 0 {
			ctx.done_with(
				"settings",
				Msg::new("settings_backed_up", format!("{moved} settings file(s) moved to {}", backup.display()))
					.with("count", moved.to_string())
					.with("path", backup.display().to_string()),
			);
			ctx.note(
				Msg::new("repair_backup", format!("Your previous settings are in {}", backup.display()))
					.with("path", backup.display().to_string()),
			);
		} else {
			ctx.skip("settings", Msg::new("no_settings", "no settings files to reset"));
		}
		let qmlcache = ctx.paths.quickshell.join("cache").join("qmlcache");
		let _ = fsops::remove_any(&qmlcache);

		ctx.log.close();
		let r = swap_in(ctx, &pkg, false);
		ctx.log.open(&ctx.paths.log());
		r?;
		m.version = pkg.version.clone();
		m.setup_version = ctx.setup_version.clone();
		m.repaired_at = Some(log::timestamp());
		seed_colors(ctx, &mut m, true);
		copy_setup_exe(ctx, &pkg);
		save(ctx, &m);
		apply_integrations(ctx, &mut m, &opts, &opts);
		ctx.begin("finish");
		m.state = "installed".into();
		save(ctx, &m);
		ctx.done("finish");
		if run.launch {
			launch(ctx);
		} else {
			ctx.skip("launch", Msg::new("launch_off", "not started"));
		}
		Ok(())
	})();
	ctx.finish(result, !run.launch)
}

/// Files that couldn't be deleted now (a font a running program holds): deleted by a hidden
/// cmd.exe a few seconds after the setup closes, together with the setup's temp folder.
fn schedule_leftovers(ctx: &mut Ctx, files: &[PathBuf]) {
	if files.is_empty() {
		return;
	}
	let list = ctx.scratch.join("delete-after-exit.txt");
	let mut text = std::fs::read_to_string(&list).unwrap_or_default();
	for f in files {
		text.push_str(&f.display().to_string());
		text.push('\n');
	}
	let _ = std::fs::write(&list, text);
}

pub fn uninstall(ctx: &mut Ctx, run: &RunOptions) -> bool {
	let Some(mut m) = Manifest::load(&ctx.paths.manifest()).ok().flatten() else {
		return ctx.finish(Err(Msg::new("not_installed", "ii-windows isn't installed by this setup")), false);
	};
	let tools: Vec<String> =
		m.items.winget.iter().map(|p| p.id.clone()).filter(|id| id != crate::PWSH_WINGET_ID).collect();
	let pwsh_ours = m.has_winget(crate::PWSH_WINGET_ID);
	let mut plan = vec!["stop", "autostart", "shortcuts", "profiles"];
	if m.items.exec_policy.as_ref().is_some_and(|c| c.changed) {
		plan.push("exec_policy");
	}
	if (run.remove_tools && !tools.is_empty()) || (run.remove_pwsh7 && pwsh_ours) {
		plan.push("tools");
	}
	plan.extend(["fonts", "terminal_theme"]);
	if run.restore_look {
		plan.push("restore");
	}
	plan.extend(["ii_data", "files"]);
	ctx.plan(&plan);
	// The install dir (and setup.log in it) is about to go: this run logs to the temp folder.
	let log_path = ctx.paths.temp.join(format!("ii-windows-uninstall-{}.log", log::stamp()));
	ctx.log.open(&log_path);
	ctx.info(format!("ii-windows setup {} - uninstall {} ({:?})", ctx.setup_version, m.version, run));
	let backup_root = ctx.paths.temp.join(format!("ii-windows-uninstall-backup-{}", log::stamp()));

	let result = (|| -> R<()> {
		stop_ii(ctx, Some(&m));

		set_autostart(ctx, &mut m, false);
		save(ctx, &m);

		ctx.begin("shortcuts");
		for s in std::mem::take(&mut m.items.shortcuts) {
			if let Err(e) = fsops::remove_any(&s) {
				ctx.warn("shortcuts", io_msg("remove_failed", s.display(), e));
			}
		}
		if let Some(key) = m.items.uninstall_key.clone() {
			// Removed last, with the files; until then a failed uninstall can be retried from
			// Apps & features.
			ctx.info(format!("   Apps & features entry {key} is removed at the end"));
		}
		save(ctx, &m);
		ctx.done("shortcuts");

		ctx.begin("profiles");
		let had_profiles = !m.items.profiles.is_empty();
		remove_profile_blocks(ctx, &mut m, "profiles", &backup_root);
		save(ctx, &m);
		if had_profiles {
			ctx.done("profiles");
			if backup_root.exists() {
				ctx.note(
					Msg::new(
						"profile_backup",
						format!(
							"Copies of your PowerShell profiles from before the block was removed: {}",
							backup_root.display()
						),
					)
					.with("path", backup_root.display().to_string()),
				);
			}
		}

		if let Some(change) = m.items.exec_policy.clone().filter(|c| c.changed) {
			ctx.begin("exec_policy");
			restore_exec_policy(ctx, &change, "exec_policy");
			m.items.exec_policy = None;
			save(ctx, &m);
		}

		if plan.contains(&"tools") {
			ctx.begin("tools");
			match win::winget_exe() {
				None => ctx.warn(
					"tools",
					Msg::new(
						"winget_missing_uninstall",
						"winget was not found: the terminal tools were left installed",
					),
				),
				Some(winget) => {
					let mut ids: Vec<String> = if run.remove_tools { tools.clone() } else { Vec::new() };
					if run.remove_pwsh7 && pwsh_ours {
						ids.push(crate::PWSH_WINGET_ID.into());
					}
					let mut failed = false;
					for (i, id) in ids.iter().enumerate() {
						ctx.progress("tools", i as f64 / ids.len() as f64, Some(id.clone()));
						ctx.info(format!("   winget uninstall {id}"));
						match win::winget_uninstall(&winget, id) {
							Ok(out) => {
								ctx.info(format!("   {}", out.text().replace('\n', "\n   ")));
								m.items.winget.retain(|p| !p.id.eq_ignore_ascii_case(id));
								save(ctx, &m);
							}
							Err(out) => {
								failed = true;
								ctx.info(format!("   {}", out.text().replace('\n', "\n   ")));
								ctx.warn(
									"tools",
									Msg::new(
										"tool_uninstall_failed",
										format!("winget couldn't uninstall {id} (exit {:?})", out.code),
									)
									.with("id", id.clone()),
								);
							}
						}
					}
					ctx.progress("tools", 1.0, None);
					if !failed {
						ctx.done("tools");
					}
				}
			}
		}

		ctx.begin("fonts");
		let had_fonts = !m.items.fonts.is_empty();
		let left = remove_fonts(ctx, &mut m, "fonts");
		schedule_leftovers(ctx, &left);
		save(ctx, &m);
		if had_fonts {
			if left.is_empty() {
				ctx.done("fonts");
			} else {
				ctx.done_with(
					"fonts",
					Msg::new(
						"fonts_in_use",
						format!("{} font file(s) were in use and are deleted after this window closes", left.len()),
					)
					.with("count", left.len().to_string()),
				);
			}
		}

		ctx.begin("terminal_theme");
		if ctx.paths.wt_fragment.exists() {
			match fsops::remove_any(&ctx.paths.wt_fragment) {
				Ok(()) => ctx.done_with(
					"terminal_theme",
					Msg::new("removed_path", format!("removed {}", ctx.paths.wt_fragment.display()))
						.with("path", ctx.paths.wt_fragment.display().to_string()),
				),
				Err(e) => ctx.warn("terminal_theme", io_msg("remove_failed", ctx.paths.wt_fragment.display(), e)),
			}
		} else {
			ctx.skip("terminal_theme", Msg::new("nothing_to_remove", "nothing there"));
		}

		if run.restore_look {
			ctx.begin("restore");
			restore_look(ctx, &m);
		}

		ctx.begin("ii_data");
		let mut dirs = ctx.paths.quickshell_dirs();
		dirs.push(ctx.paths.ii_temp());
		dirs.push(ctx.paths.quickshell.join("ii.new"));
		dirs.push(ctx.paths.quickshell.join("ii.old"));
		if !run.keep_settings {
			dirs.push(ctx.paths.settings.clone());
		}
		let mut failed = false;
		for d in &dirs {
			if d.exists() {
				match fsops::remove_any(d) {
					Ok(()) => ctx.info(format!("   removed {}", d.display())),
					Err(e) => {
						failed = true;
						ctx.warn("ii_data", io_msg("remove_failed", d.display(), e));
					}
				}
			}
		}
		// %LOCALAPPDATA%\quickshell itself only if nothing else (another Quickshell config) is
		// left in it.
		if std::fs::read_dir(&ctx.paths.quickshell).is_ok_and(|mut rd| rd.next().is_none()) {
			let _ = std::fs::remove_dir(&ctx.paths.quickshell);
		}
		if run.keep_settings && ctx.paths.settings.exists() {
			ctx.note(
				Msg::new("settings_kept", format!("Your settings were kept in {}", ctx.paths.settings.display()))
					.with("path", ctx.paths.settings.display().to_string()),
			);
		}
		if !failed {
			ctx.done("ii_data");
		}

		ctx.begin("files");
		let _ = fsops::remove_any(&ctx.paths.staging());
		let _ = fsops::remove_any(&ctx.paths.previous());
		if let Err(e) = fsops::remove_any(&ctx.paths.install_dir) {
			let m2 = io_msg("files_in_use", ctx.paths.install_dir.display(), e);
			ctx.fail("files", &m2);
			return Err(m2);
		}
		if let Some(key) = m.items.uninstall_key.take() {
			let rel = key.strip_prefix("HKCU\\").unwrap_or(&key).to_string();
			if let Err(e) = win::delete_tree(&rel) {
				ctx.warn("files", io_msg("remove_failed", &key, e));
			}
		}
		ctx.done("files");
		ctx.note(
			Msg::new("uninstall_log", format!("The uninstall log is {}", log_path.display()))
				.with("path", log_path.display().to_string()),
		);
		Ok(())
	})();
	ctx.finish(result, false)
}

fn restore_look(ctx: &mut Ctx, m: &Manifest) {
	let Some(pre) = m.pre_install.clone() else {
		ctx.skip("restore", Msg::new("nothing_recorded", "nothing was recorded at install time"));
		return;
	};
	// Taskbar first: ii is closed now, so whatever it is now is what Windows will keep.
	win::show_taskbars();
	if let Some(autohide) = pre.taskbar_autohide {
		if win::taskbar_autohide() != Some(autohide) {
			win::set_taskbar_autohide(autohide);
			ctx.info(format!("   taskbar auto-hide back to {autohide}"));
		}
	}
	win::show_taskbars();
	let mut problems = Vec::new();
	let regs = [
		(win::PERSONALIZE_KEY, "AppsUseLightTheme", pre.apps_use_light_theme),
		(win::PERSONALIZE_KEY, "SystemUsesLightTheme", pre.system_uses_light_theme),
		(win::DWM_KEY, "AccentColor", pre.accent_color),
		(win::DWM_KEY, "ColorizationColor", pre.colorization_color),
		(win::DWM_KEY, "ColorizationAfterglow", pre.colorization_afterglow),
	];
	for (key, name, value) in regs {
		let now = win::get_dword(key, name);
		if now != value {
			match win::restore_dword(key, name, value) {
				Ok(()) => ctx.info(format!("   {name}: {now:?} -> {value:?}")),
				Err(e) => problems.push(format!("{name}: {e}")),
			}
		}
	}
	win::broadcast_setting_change("ImmersiveColorSet");
	if let Some(w) = &pre.wallpaper {
		match win::restore_wallpaper(w, &ctx.paths.install_dir, &ctx.paths.roaming, &ctx.scratch.join("wallpaper")) {
			Ok(notes) => {
				for n in notes {
					if n == "slideshow/spotlight" {
						ctx.note(Msg::new("slideshow_not_restored", "Your background was a slideshow or Windows spotlight: the picture showing at install time is back, turn the slideshow/spotlight on again in Settings > Personalization > Background"));
					} else {
						problems.push(n);
					}
				}
			}
			Err(e) => problems.push(format!("wallpaper: {e}")),
		}
	}
	if problems.is_empty() {
		ctx.done("restore");
	} else {
		let text = problems.join("; ");
		ctx.warn(
			"restore",
			Msg::new("restore_partial", format!("Some of the look couldn't be restored: {text}")).with("error", text),
		);
	}
	if pre.other_instance_running {
		ctx.note(Msg::new("restore_uncertain", "Another copy of ii was running when the installer recorded your taskbar settings, so they may not be exactly yours"));
	}
}

/// Run by the app when its window closes: deletes this run's temp folder (and any file listed
/// for deletion) from a hidden cmd.exe once the setup and WebView2 have exited.
pub fn schedule_cleanup(scratch: &Path, extra_dirs: &[PathBuf]) {
	let mut targets: Vec<String> = Vec::new();
	if let Ok(text) = std::fs::read_to_string(scratch.join("delete-after-exit.txt")) {
		for l in text.lines().filter(|l| !l.trim().is_empty()) {
			targets.push(format!("del /f /q \"{}\" 2>nul", l.trim()));
		}
	}
	for d in std::iter::once(&scratch.to_path_buf()).chain(extra_dirs.iter()) {
		targets.push(format!("rmdir /s /q \"{}\" 2>nul", d.display()));
	}
	let body = targets.join(" & ");
	let check = format!("if not exist \"{}\" exit", scratch.display());
	// Up to ~20 s: WebView2's processes let go of their data folder a moment after we exit.
	let line = format!("for /l %i in (1,1,10) do (ping -n 3 127.0.0.1 >nul & {body} & {check})");
	let _ = win::spawn_cmd_raw(&line);
}
