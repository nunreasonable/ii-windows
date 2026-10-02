//! The release package: `ii-windows-<version>.zip`, laid out like `dist/ii-windows` (qs.exe,
//! qsw.exe, the Qt runtime, fonts\, icons\, qml\, config\ii\, ...) plus a VERSION file and,
//! when built by tools/release.sh, the setup exe itself.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::version;

/// Files a package must have for the installer to accept it.
pub const REQUIRED: [&str; 3] = ["qsw.exe", "qs.exe", "config/ii/shell.qml"];

#[derive(Debug, Clone, serde::Serialize)]
pub struct PackageInfo {
	pub path: PathBuf,
	pub version: String,
	pub file_size: u64,
	pub unpacked_size: u64,
	pub files: usize,
	/// "ii-windows/" when everything sits in one top-level folder, else "".
	#[serde(skip)]
	pub prefix: String,
	pub has_setup_exe: bool,
}

#[derive(Debug)]
pub enum Error {
	Io(std::io::Error),
	Zip(String),
	Invalid(String),
	Cancelled,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Io(e) => write!(f, "{e}"),
			Error::Zip(e) => write!(f, "the package is not a readable zip: {e}"),
			Error::Invalid(e) => write!(f, "{e}"),
			Error::Cancelled => write!(f, "cancelled"),
		}
	}
}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Self {
		Error::Io(e)
	}
}

impl From<zip::result::ZipError> for Error {
	fn from(e: zip::result::ZipError) -> Self {
		Error::Zip(e.to_string())
	}
}

fn common_prefix(names: &[String]) -> String {
	let Some(first) = names.first() else { return String::new() };
	let Some((top, _)) = first.split_once('/') else { return String::new() };
	let prefix = format!("{top}/");
	if names.iter().all(|n| n.starts_with(&prefix) || n == top)
		&& names.iter().any(|n| n == &format!("{prefix}qsw.exe"))
	{
		prefix
	} else {
		String::new()
	}
}

/// Reads the zip's directory (no extraction) and checks it's an ii-windows package.
pub fn inspect(path: &Path) -> Result<PackageInfo, Error> {
	let file = File::open(path)?;
	let file_size = file.metadata()?.len();
	let mut zip = zip::ZipArchive::new(file)?;
	let names: Vec<String> = zip.file_names().map(|n| n.replace('\\', "/")).collect();
	let prefix = common_prefix(&names);
	for req in REQUIRED {
		let want = format!("{prefix}{req}");
		if !names.iter().any(|n| n.eq_ignore_ascii_case(&want)) {
			return Err(Error::Invalid(format!("the package has no {req}; it is not an ii-windows package")));
		}
	}
	let mut unpacked = 0u64;
	let mut files = 0usize;
	for i in 0..zip.len() {
		let f = zip.by_index_raw(i)?;
		if !f.is_dir() {
			unpacked += f.size();
			files += 1;
		}
	}
	let mut ver = String::new();
	if let Ok(f) = zip.by_name(&format!("{prefix}VERSION")) {
		let mut s = String::new();
		f.take(64).read_to_string(&mut s).ok();
		ver = version::normalize(s.trim());
	}
	if ver.is_empty() {
		let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
		ver = version::from_package_name(name).unwrap_or_default();
	}
	if ver.is_empty() {
		return Err(Error::Invalid(
			"the package has no VERSION file and its name isn't ii-windows-<version>.zip".into(),
		));
	}
	let has_setup_exe = names.iter().any(|n| n.eq_ignore_ascii_case(&format!("{prefix}{}", crate::SETUP_EXE)));
	Ok(PackageInfo {
		path: path.to_path_buf(),
		version: ver,
		file_size,
		unpacked_size: unpacked,
		files,
		prefix,
		has_setup_exe,
	})
}

/// Unpacks into `dest` (which must not exist yet), calling `progress(bytes_done, bytes_total)`.
pub fn extract(
	info: &PackageInfo,
	dest: &Path,
	progress: &mut dyn FnMut(u64, u64),
	cancel: &dyn Fn() -> bool,
) -> Result<(), Error> {
	let mut zip = zip::ZipArchive::new(File::open(&info.path)?)?;
	std::fs::create_dir_all(dest)?;
	let mut done = 0u64;
	let mut buf = vec![0u8; 256 * 1024];
	for i in 0..zip.len() {
		if cancel() {
			return Err(Error::Cancelled);
		}
		let mut entry = zip.by_index(i)?;
		// enclosed_name() refuses absolute paths and "..": nothing lands outside `dest`.
		let Some(rel) = entry.enclosed_name() else {
			return Err(Error::Invalid(format!("unsafe path in package: {}", entry.name())));
		};
		let rel_str = rel.to_string_lossy().replace('\\', "/");
		let rel_str = rel_str.strip_prefix(&info.prefix).unwrap_or(&rel_str).to_string();
		if rel_str.is_empty() {
			continue;
		}
		let out_path = dest.join(rel_str.replace('/', std::path::MAIN_SEPARATOR_STR));
		if entry.is_dir() {
			std::fs::create_dir_all(&out_path)?;
			continue;
		}
		if let Some(parent) = out_path.parent() {
			std::fs::create_dir_all(parent)?;
		}
		let mut out = File::create(&out_path)?;
		loop {
			let n = entry.read(&mut buf).map_err(|e| Error::Zip(format!("{}: {e}", entry.name())))?;
			if n == 0 {
				break;
			}
			out.write_all(&buf[..n])?;
			done += n as u64;
			progress(done, info.unpacked_size);
		}
	}
	for req in REQUIRED {
		if !dest.join(req.replace('/', std::path::MAIN_SEPARATOR_STR)).is_file() {
			return Err(Error::Invalid(format!("{req} is missing after unpacking")));
		}
	}
	Ok(())
}

/// The newest `ii-windows-<version>.zip` in `dir`, if any (the offline source next to the exe).
pub fn find_offline(dir: &Path) -> Option<PathBuf> {
	let mut best: Option<(version::Version, PathBuf)> = None;
	for entry in std::fs::read_dir(dir).ok()?.flatten() {
		let name = entry.file_name().to_string_lossy().to_string();
		let Some(v) = version::from_package_name(&name).and_then(|v| version::Version::parse(&v)) else { continue };
		if !entry.path().is_file() {
			continue;
		}
		if best.as_ref().is_none_or(|(b, _)| v > *b) {
			best = Some((v, entry.path()));
		}
	}
	best.map(|(_, p)| p)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn make_zip(path: &Path, prefix: &str, files: &[(&str, &[u8])]) {
		let mut w = zip::ZipWriter::new(File::create(path).unwrap());
		let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
		for (name, data) in files {
			w.start_file(format!("{prefix}{name}"), opts).unwrap();
			w.write_all(data).unwrap();
		}
		w.finish().unwrap();
	}

	fn tmp(name: &str) -> PathBuf {
		let d = std::env::temp_dir().join(format!("iiw-pkg-{}-{name}", std::process::id()));
		let _ = std::fs::remove_dir_all(&d);
		std::fs::create_dir_all(&d).unwrap();
		d
	}

	#[test]
	fn inspect_and_extract() {
		let d = tmp("a");
		let zip_path = d.join("ii-windows-0.3.0.zip");
		let files: &[(&str, &[u8])] = &[
			("qsw.exe", b"MZ"),
			("qs.exe", b"MZ"),
			("config/ii/shell.qml", b"import QtQuick"),
			("fonts/JetBrainsMonoNerdFont-Regular.ttf", b"font"),
			("VERSION", b"0.3.1\n"),
		];
		make_zip(&zip_path, "", files);
		let info = inspect(&zip_path).unwrap();
		assert_eq!(info.version, "0.3.1", "VERSION wins over the file name");
		assert_eq!(info.files, 5);
		assert!(!info.has_setup_exe);
		let out = d.join("out");
		let mut last = 0;
		extract(&info, &out, &mut |done, _| last = done, &|| false).unwrap();
		assert_eq!(last, info.unpacked_size);
		assert_eq!(std::fs::read(out.join("config").join("ii").join("shell.qml")).unwrap(), b"import QtQuick");
		assert_eq!(find_offline(&d).unwrap(), zip_path);
		std::fs::remove_dir_all(&d).unwrap();
	}

	#[test]
	fn single_top_folder_and_rejects() {
		let d = tmp("b");
		let zip_path = d.join("ii-windows-0.1.0.zip");
		make_zip(&zip_path, "ii-windows/", &[("qsw.exe", b"MZ"), ("qs.exe", b"MZ"), ("config/ii/shell.qml", b"x")]);
		let info = inspect(&zip_path).unwrap();
		assert_eq!(info.prefix, "ii-windows/");
		assert_eq!(info.version, "0.1.0");
		let out = d.join("out");
		extract(&info, &out, &mut |_, _| {}, &|| false).unwrap();
		assert!(out.join("qsw.exe").is_file());

		let bad = d.join("ii-windows-0.2.0.zip");
		make_zip(&bad, "", &[("readme.txt", b"x")]);
		assert!(matches!(inspect(&bad), Err(Error::Invalid(_))));
		std::fs::write(d.join("ii-windows-9.9.9.zip"), b"not a zip").unwrap();
		assert!(matches!(inspect(&d.join("ii-windows-9.9.9.zip")), Err(Error::Zip(_))));
		std::fs::remove_dir_all(&d).unwrap();
	}
}
