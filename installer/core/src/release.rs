use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use crate::arch::Arch;
use crate::version;

pub const REPO: &str = "nunreasonable/ii-windows";
pub const REPO_URL: &str = "https://github.com/nunreasonable/ii-windows";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReleaseInfo {
	pub version: String,
	pub tag: String,
	pub html_url: String,
	pub published_at: String,
	pub zip_name: String,
	pub zip_url: String,
	pub zip_size: u64,
	pub sha_url: String,
	pub setup_url: String,
	pub setup_size: u64,
	pub setup_sha256: String,
	pub arch: Arch,
	pub wanted_arch: Arch,
}

#[derive(Debug)]
pub enum Error {
	NotFound,
	Network(String),
	NoPackage(String),
	Integrity(String),
	Io(std::io::Error),
	Cancelled,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::NotFound => write!(f, "no release found on GitHub ({REPO})"),
			Error::Network(e) => write!(f, "network error: {e}"),
			Error::NoPackage(tag) => {
				write!(f, "release {tag} has no ii-windows-<version>.zip with a .sha256 next to it")
			}
			Error::Integrity(e) => write!(f, "{e}"),
			Error::Io(e) => write!(f, "{e}"),
			Error::Cancelled => write!(f, "cancelled"),
		}
	}
}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Self {
		Error::Io(e)
	}
}

fn agent() -> ureq::Agent {
	ureq::Agent::config_builder()
		.timeout_connect(Some(Duration::from_secs(15)))
		.timeout_recv_response(Some(Duration::from_secs(30)))
		.user_agent(concat!("ii-windows-setup/", env!("CARGO_PKG_VERSION")))
		.http_status_as_error(false)
		.build()
		.into()
}

fn get_json(url: &str) -> Result<serde_json::Value, Error> {
	let mut resp = agent()
		.get(url)
		.header("Accept", "application/vnd.github+json")
		.header("X-GitHub-Api-Version", "2022-11-28")
		.call()
		.map_err(|e| Error::Network(e.to_string()))?;
	let status = resp.status().as_u16();
	if status == 404 {
		return Err(Error::NotFound);
	}
	if status != 200 {
		let body = resp.body_mut().read_to_string().unwrap_or_default();
		let msg = serde_json::from_str::<serde_json::Value>(&body)
			.ok()
			.and_then(|v| v.get("message").and_then(|m| m.as_str()).map(String::from))
			.unwrap_or_default();
		return Err(Error::Network(format!("GitHub answered HTTP {status} {msg}").trim().to_string()));
	}
	let body = resp.body_mut().read_to_string().map_err(|e| Error::Network(e.to_string()))?;
	serde_json::from_str(&body).map_err(|e| Error::Network(format!("bad JSON from GitHub: {e}")))
}

pub fn parse_release(json: &serde_json::Value, wanted_arch: Arch) -> Result<ReleaseInfo, Error> {
	let tag = json.get("tag_name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
	let ver = version::normalize(&tag);
	let assets = json.get("assets").and_then(|a| a.as_array()).cloned().unwrap_or_default();
	let name_of = |a: &serde_json::Value| a.get("name").and_then(|n| n.as_str()).unwrap_or_default().to_string();
	let find = |name: &str| assets.iter().find(|a| name_of(a).eq_ignore_ascii_case(name));
	let pick = |arch: Arch| {
		let zip = find(&arch.package_name(&ver)).or_else(|| {
			assets.iter().find(|a| version::package_name_parts(&name_of(a)).is_some_and(|(_, found)| found == arch))
		})?;
		let sha = find(&format!("{}.sha256", name_of(zip)))?;
		Some((zip, sha))
	};
	let chosen = match pick(wanted_arch) {
		Some(found) => Some((found, wanted_arch)),
		None if wanted_arch != Arch::X64 => pick(Arch::X64).map(|found| (found, Arch::X64)),
		None => None,
	};
	let Some(((zip, sha), arch)) = chosen else { return Err(Error::NoPackage(tag)) };
	let zip_name = name_of(zip);
	let url =
		|a: &serde_json::Value| a.get("browser_download_url").and_then(|v| v.as_str()).unwrap_or_default().to_string();
	let version = version::from_package_name(&zip_name).filter(|_| ver.is_empty()).unwrap_or(ver);
	let setup = find(crate::SETUP_EXE);
	let setup_sha256 = setup
		.and_then(|a| a.get("digest"))
		.and_then(|v| v.as_str())
		.and_then(|d| d.strip_prefix("sha256:"))
		.map(str::to_ascii_lowercase)
		.unwrap_or_default();
	Ok(ReleaseInfo {
		version,
		tag: tag.clone(),
		html_url: json.get("html_url").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
		published_at: json.get("published_at").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
		zip_name,
		zip_url: url(zip),
		zip_size: zip.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
		sha_url: url(sha),
		setup_url: setup.map(url).unwrap_or_default(),
		setup_size: setup.and_then(|a| a.get("size")).and_then(|v| v.as_u64()).unwrap_or(0),
		setup_sha256,
		arch,
		wanted_arch,
	})
}

pub const RELEASE_JSON_ENV: &str = "IIW_SETUP_RELEASE_JSON";

fn override_release(arch: Arch) -> Option<Result<ReleaseInfo, Error>> {
	let url = std::env::var(RELEASE_JSON_ENV).ok().filter(|u| !u.trim().is_empty())?;
	Some(get_json(url.trim()).and_then(|j| parse_release(&j, arch)))
}

pub(crate) fn latest_release_json(repo: &str) -> Result<serde_json::Value, Error> {
	get_json(&format!("https://api.github.com/repos/{repo}/releases/latest"))
}

pub fn latest(arch: Arch) -> Result<ReleaseInfo, Error> {
	if let Some(r) = override_release(arch) {
		return r;
	}
	parse_release(&latest_release_json(REPO)?, arch)
}

pub fn for_version(ver: &str, arch: Arch) -> Result<ReleaseInfo, Error> {
	if let Some(r) = override_release(arch) {
		return r.and_then(|rel| if version::same(&rel.version, ver) { Ok(rel) } else { Err(Error::NotFound) });
	}
	let ver = version::normalize(ver);
	let mut last = Error::NotFound;
	for tag in [format!("v{ver}"), ver.clone()] {
		match get_json(&format!("https://api.github.com/repos/{REPO}/releases/tags/{tag}")) {
			Ok(json) => return parse_release(&json, arch),
			Err(Error::NotFound) => continue,
			Err(e) => last = e,
		}
	}
	Err(last)
}

pub fn parse_sha256_file(text: &str, file_name: &str) -> Result<String, Error> {
	let mut lone = None;
	for line in text.lines() {
		let line = line.trim();
		if line.is_empty() {
			continue;
		}
		let mut parts = line.split_whitespace();
		let hex = parts.next().unwrap_or_default();
		if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
			continue;
		}
		match parts.next() {
			Some(name) => {
				let name = name.trim_start_matches('*');
				let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
				if base.eq_ignore_ascii_case(file_name) {
					return Ok(hex.to_ascii_lowercase());
				}
			}
			None => lone = Some(hex.to_ascii_lowercase()),
		}
	}
	lone.ok_or_else(|| Error::Integrity(format!("no SHA-256 for {file_name} in the .sha256 file")))
}

pub fn sha256_file(path: &Path) -> std::io::Result<String> {
	let mut f = std::fs::File::open(path)?;
	let mut h = Sha256::new();
	let mut buf = vec![0u8; 1 << 20];
	loop {
		let n = f.read(&mut buf)?;
		if n == 0 {
			break;
		}
		h.update(&buf[..n]);
	}
	Ok(hex(&h.finalize()))
}

fn hex(bytes: &[u8]) -> String {
	bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn fetch_text(url: &str) -> Result<String, Error> {
	let mut resp = agent().get(url).call().map_err(|e| Error::Network(e.to_string()))?;
	match resp.status().as_u16() {
		200 => resp.body_mut().read_to_string().map_err(|e| Error::Network(e.to_string())),
		404 => Err(Error::NotFound),
		s => Err(Error::Network(format!("HTTP {s} for {url}"))),
	}
}

pub fn download(
	url: &str,
	dest: &Path,
	expected_size: u64,
	progress: &mut dyn FnMut(u64, u64),
	cancel: &dyn Fn() -> bool,
) -> Result<String, Error> {
	let mut resp = agent().get(url).call().map_err(|e| Error::Network(e.to_string()))?;
	let status = resp.status().as_u16();
	if status != 200 {
		return Err(Error::Network(format!("HTTP {status} downloading {url}")));
	}
	let total = resp
		.headers()
		.get("content-length")
		.and_then(|v| v.to_str().ok())
		.and_then(|v| v.parse::<u64>().ok())
		.unwrap_or(expected_size);
	if let Some(dir) = dest.parent() {
		std::fs::create_dir_all(dir)?;
	}
	let mut out = std::fs::File::create(dest)?;
	let mut reader = resp.body_mut().with_config().limit(u64::MAX).reader();
	let mut h = Sha256::new();
	let mut buf = vec![0u8; 256 * 1024];
	let mut done = 0u64;
	loop {
		if cancel() {
			return Err(Error::Cancelled);
		}
		let n = reader.read(&mut buf).map_err(|e| Error::Network(e.to_string()))?;
		if n == 0 {
			break;
		}
		out.write_all(&buf[..n])?;
		h.update(&buf[..n]);
		done += n as u64;
		progress(done, total);
	}
	out.flush()?;
	if total > 0 && done != total {
		return Err(Error::Network(format!("download ended early ({done} of {total} bytes)")));
	}
	Ok(hex(&h.finalize()))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn release_json() {
		let json: serde_json::Value = serde_json::from_str(
			r#"{
			"tag_name": "v0.2.0", "html_url": "https://github.com/x/y/releases/tag/v0.2.0",
			"published_at": "2026-10-02T00:00:00Z",
			"assets": [
				{"name": "ii-windows-setup.exe", "size": 7000000, "browser_download_url": "https://e/setup"},
				{"name": "ii-windows-0.2.0.zip", "size": 100000000, "browser_download_url": "https://e/zip"},
				{"name": "ii-windows-0.2.0.zip.sha256", "size": 90, "browser_download_url": "https://e/sha"}
			]}"#,
		)
		.unwrap();
		let r = parse_release(&json, Arch::X64).unwrap();
		assert_eq!(r.version, "0.2.0");
		assert_eq!(r.zip_url, "https://e/zip");
		assert_eq!(r.sha_url, "https://e/sha");
		assert_eq!(r.zip_size, 100000000);

		let no_sha: serde_json::Value = serde_json::from_str(
			r#"{"tag_name": "v0.2.0", "assets": [
			{"name": "ii-windows-0.2.0.zip", "browser_download_url": "https://e/zip"}]}"#,
		)
		.unwrap();
		assert!(matches!(parse_release(&no_sha, Arch::X64), Err(Error::NoPackage(_))));
	}

	fn release(assets: &[&str]) -> serde_json::Value {
		let assets: Vec<serde_json::Value> = assets
			.iter()
			.map(|n| serde_json::json!({"name": n, "size": n.len(), "browser_download_url": format!("https://e/{n}")}))
			.collect();
		serde_json::json!({"tag_name": "v26.1.2.0", "assets": assets})
	}

	const BOTH: [&str; 5] = [
		"ii-windows-setup.exe",
		"ii-windows-26.1.2.0.zip",
		"ii-windows-26.1.2.0.zip.sha256",
		"ii-windows-26.1.2.0-arm64.zip",
		"ii-windows-26.1.2.0-arm64.zip.sha256",
	];

	#[test]
	fn x64_takes_the_plain_zip() {
		let r = parse_release(&release(&BOTH), Arch::X64).unwrap();
		assert_eq!(r.zip_name, "ii-windows-26.1.2.0.zip");
		assert_eq!(r.sha_url, "https://e/ii-windows-26.1.2.0.zip.sha256");
		assert_eq!(r.version, "26.1.2.0");
		assert_eq!((r.arch, r.wanted_arch), (Arch::X64, Arch::X64));
		assert_eq!(r.setup_url, "https://e/ii-windows-setup.exe");

		let reversed: Vec<&str> = BOTH.iter().rev().copied().collect();
		assert_eq!(parse_release(&release(&reversed), Arch::X64).unwrap().zip_name, "ii-windows-26.1.2.0.zip");

		let only_arm = release(&["ii-windows-26.1.2.0-arm64.zip", "ii-windows-26.1.2.0-arm64.zip.sha256"]);
		assert!(
			matches!(parse_release(&only_arm, Arch::X64), Err(Error::NoPackage(_))),
			"x64 never takes the arm64 zip"
		);
	}

	#[test]
	fn arm64_takes_the_arm64_zip() {
		let r = parse_release(&release(&BOTH), Arch::Arm64).unwrap();
		assert_eq!(r.zip_name, "ii-windows-26.1.2.0-arm64.zip");
		assert_eq!(r.zip_url, "https://e/ii-windows-26.1.2.0-arm64.zip");
		assert_eq!(r.sha_url, "https://e/ii-windows-26.1.2.0-arm64.zip.sha256");
		assert_eq!(r.version, "26.1.2.0");
		assert_eq!((r.arch, r.wanted_arch), (Arch::Arm64, Arch::Arm64));
		assert_eq!(r.setup_url, "https://e/ii-windows-setup.exe");

		let renamed = release(&[
			"ii-windows-26.1.2.0.zip",
			"ii-windows-26.1.2.0.zip.sha256",
			"ii-windows-26.1.2.0-fix1-arm64.zip",
			"ii-windows-26.1.2.0-fix1-arm64.zip.sha256",
		]);
		assert_eq!(parse_release(&renamed, Arch::Arm64).unwrap().zip_name, "ii-windows-26.1.2.0-fix1-arm64.zip");
	}

	#[test]
	fn arm64_falls_back_to_x64() {
		let old = release(&["ii-windows-setup.exe", "ii-windows-26.1.2.0.zip", "ii-windows-26.1.2.0.zip.sha256"]);
		let r = parse_release(&old, Arch::Arm64).unwrap();
		assert_eq!(r.zip_name, "ii-windows-26.1.2.0.zip");
		assert_eq!((r.arch, r.wanted_arch), (Arch::X64, Arch::Arm64));

		let arm_without_sha =
			release(&["ii-windows-26.1.2.0.zip", "ii-windows-26.1.2.0.zip.sha256", "ii-windows-26.1.2.0-arm64.zip"]);
		let r = parse_release(&arm_without_sha, Arch::Arm64).unwrap();
		assert_eq!(r.zip_name, "ii-windows-26.1.2.0.zip");
		assert_eq!((r.arch, r.wanted_arch), (Arch::X64, Arch::Arm64));

		let nothing = release(&["ii-windows-setup.exe"]);
		assert!(matches!(parse_release(&nothing, Arch::Arm64), Err(Error::NoPackage(_))));
	}

	#[test]
	fn update_keeps_the_installed_arch() {
		let path = std::env::temp_dir().join(format!("iiw-release-arch-{}.json", std::process::id()));
		let pick = |manifest: &str, native: Arch| {
			std::fs::write(&path, manifest).unwrap();
			let m = crate::manifest::Manifest::load(&path).unwrap().unwrap();
			parse_release(&release(&BOTH), m.update_arch(native)).unwrap().zip_name
		};
		assert_eq!(
			pick(r#"{"version":"26.1.1.0","arch":"arm64","wanted_arch":"arm64"}"#, Arch::Arm64),
			"ii-windows-26.1.2.0-arm64.zip"
		);
		assert_eq!(
			pick(r#"{"version":"26.1.1.0","arch":"x64","wanted_arch":"x64"}"#, Arch::Arm64),
			"ii-windows-26.1.2.0.zip"
		);
		assert_eq!(
			pick(r#"{"version":"26.1.1.0","arch":"x64","wanted_arch":"arm64"}"#, Arch::Arm64),
			"ii-windows-26.1.2.0-arm64.zip"
		);
		assert_eq!(pick(r#"{"version":"26.1.1.0"}"#, Arch::Arm64), "ii-windows-26.1.2.0.zip");
		assert_eq!(
			pick(r#"{"version":"26.1.1.0","arch":"arm64","wanted_arch":"arm64"}"#, Arch::X64),
			"ii-windows-26.1.2.0.zip"
		);
		std::fs::remove_file(&path).unwrap();
	}

	#[test]
	fn sha_files() {
		let h = "a".repeat(64);
		assert_eq!(parse_sha256_file(&format!("{h}  ii-windows-0.1.0.zip\n"), "ii-windows-0.1.0.zip").unwrap(), h);
		assert_eq!(
			parse_sha256_file(&format!("{h} *dist/release/ii-windows-0.1.0.zip"), "ii-windows-0.1.0.zip").unwrap(),
			h
		);
		assert_eq!(parse_sha256_file(&format!("{}\n", h.to_uppercase()), "x.zip").unwrap(), h);
		assert!(parse_sha256_file(&format!("{h}  other.zip"), "ii-windows-0.1.0.zip").is_err());
		assert!(parse_sha256_file("nothing", "x").is_err());
	}

	#[test]
	fn hashing() {
		let p = std::env::temp_dir().join(format!("iiw-sha-{}", std::process::id()));
		std::fs::write(&p, b"abc").unwrap();
		assert_eq!(sha256_file(&p).unwrap(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
		std::fs::remove_file(&p).unwrap();
	}
}
