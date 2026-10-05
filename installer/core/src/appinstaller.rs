use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::release::{self, Error};

pub const REPO: &str = "microsoft/winget-cli";

const MSIXBUNDLE_NAME: &str = "Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle";
const DEPENDENCIES_ZIP_NAME: &str = "DesktopAppInstaller_Dependencies.zip";
const SHA_TXT_NAME: &str = "Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.txt";

#[derive(Debug, Clone, Serialize)]
pub struct Asset {
	pub name: String,
	pub url: String,
	pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WingetCliRelease {
	pub tag: String,
	pub html_url: String,
	pub msixbundle: Asset,
	pub dependencies_zip: Asset,
	pub sha_txt: Option<Asset>,
}

fn asset(json: &serde_json::Value, name: &str) -> Option<Asset> {
	let a = json
		.get("assets")?
		.as_array()?
		.iter()
		.find(|a| a.get("name").and_then(|n| n.as_str()).is_some_and(|n| n.eq_ignore_ascii_case(name)))?;
	Some(Asset {
		name: a.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
		url: a.get("browser_download_url").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
		size: a.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
	})
}

pub fn parse_release(json: &serde_json::Value) -> Result<WingetCliRelease, Error> {
	let tag = json.get("tag_name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
	let html_url = json.get("html_url").and_then(|v| v.as_str()).unwrap_or_default().to_string();
	let msixbundle = asset(json, MSIXBUNDLE_NAME).ok_or_else(|| Error::NoPackage(tag.clone()))?;
	let dependencies_zip = asset(json, DEPENDENCIES_ZIP_NAME).ok_or_else(|| Error::NoPackage(tag.clone()))?;
	let sha_txt = asset(json, SHA_TXT_NAME);
	Ok(WingetCliRelease { tag, html_url, msixbundle, dependencies_zip, sha_txt })
}

pub fn latest() -> Result<WingetCliRelease, Error> {
	parse_release(&release::latest_release_json(REPO)?)
}

pub fn extract_x64_dependencies(zip_path: &Path, dest: &Path) -> Result<Vec<PathBuf>, Error> {
	let file = std::fs::File::open(zip_path)?;
	let mut zip = zip::ZipArchive::new(file).map_err(|e| Error::Integrity(format!("dependencies zip: {e}")))?;
	std::fs::create_dir_all(dest)?;
	let mut out = Vec::new();
	for i in 0..zip.len() {
		let mut entry = zip.by_index(i).map_err(|e| Error::Integrity(format!("dependencies zip: {e}")))?;
		if entry.is_dir() {
			continue;
		}
		let name = entry.name().replace('\\', "/");
		let bytes = name.as_bytes();
		if bytes.len() < 4 || !bytes[..4].eq_ignore_ascii_case(b"x64/") {
			continue;
		}
		let base = &name[4..];
		if base.is_empty() || base.contains('/') {
			continue;
		}
		let out_path = dest.join(base);
		let mut f = std::fs::File::create(&out_path)?;
		std::io::copy(&mut entry, &mut f)?;
		out.push(out_path);
	}
	if out.is_empty() {
		return Err(Error::Integrity("the dependencies zip has no x64 packages".into()));
	}
	Ok(out)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn release_json() {
		let json: serde_json::Value = serde_json::from_str(
			r#"{
			"tag_name": "v1.29.380", "html_url": "https://github.com/microsoft/winget-cli/releases/tag/v1.29.380",
			"assets": [
				{"name": "DesktopAppInstaller_Dependencies.json", "size": 322, "browser_download_url": "https://e/deps.json"},
				{"name": "DesktopAppInstaller_Dependencies.zip", "size": 97760717, "browser_download_url": "https://e/deps.zip"},
				{"name": "Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle", "size": 217276577, "browser_download_url": "https://e/bundle"},
				{"name": "Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.txt", "size": 64, "browser_download_url": "https://e/bundle.txt"}
			]}"#,
		)
		.unwrap();
		let r = parse_release(&json).unwrap();
		assert_eq!(r.tag, "v1.29.380");
		assert_eq!(r.msixbundle.url, "https://e/bundle");
		assert_eq!(r.msixbundle.size, 217276577);
		assert_eq!(r.dependencies_zip.url, "https://e/deps.zip");
		assert_eq!(r.sha_txt.unwrap().url, "https://e/bundle.txt");

		let no_txt: serde_json::Value = serde_json::from_str(
			r#"{"tag_name": "v1.0.0", "assets": [
			{"name": "Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle", "size": 1, "browser_download_url": "https://e/bundle"},
			{"name": "DesktopAppInstaller_Dependencies.zip", "size": 1, "browser_download_url": "https://e/deps.zip"}]}"#,
		)
		.unwrap();
		assert!(parse_release(&no_txt).unwrap().sha_txt.is_none());

		let missing_bundle: serde_json::Value =
			serde_json::from_str(r#"{"tag_name": "v1.0.0", "assets": []}"#).unwrap();
		assert!(matches!(parse_release(&missing_bundle), Err(Error::NoPackage(_))));
	}

	fn tmp(name: &str) -> PathBuf {
		let d = std::env::temp_dir().join(format!("iiw-appinstaller-test-{}-{name}", std::process::id()));
		let _ = std::fs::remove_dir_all(&d);
		std::fs::create_dir_all(&d).unwrap();
		d
	}

	fn make_zip(path: &Path, files: &[(&str, &[u8])]) {
		use std::io::Write;
		let mut w = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
		let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
		for (name, data) in files {
			w.start_file(*name, opts).unwrap();
			w.write_all(data).unwrap();
		}
		w.finish().unwrap();
	}

	#[test]
	fn only_x64_is_extracted() {
		let d = tmp("x64");
		let zip_path = d.join("deps.zip");
		make_zip(
			&zip_path,
			&[
				("arm64/vclibs_arm64.appx", b"arm64"),
				("x64/vclibs_x64.appx", b"x64 one"),
				("x64/runtime_x64.appx", b"x64 two"),
				("x86/vclibs_x86.appx", b"x86"),
			],
		);
		let out = extract_x64_dependencies(&zip_path, &d.join("out")).unwrap();
		assert_eq!(out.len(), 2);
		for p in &out {
			assert!(p.is_file());
			assert!(!p.display().to_string().contains("x86"));
			assert!(!p.display().to_string().contains("arm64"));
		}
		std::fs::remove_dir_all(&d).unwrap();
	}

	#[test]
	fn no_x64_folder_is_an_error() {
		let d = tmp("none");
		let zip_path = d.join("deps.zip");
		make_zip(&zip_path, &[("x86/vclibs_x86.appx", b"x86")]);
		assert!(matches!(extract_x64_dependencies(&zip_path, &d.join("out")), Err(Error::Integrity(_))));
		std::fs::remove_dir_all(&d).unwrap();
	}
}
