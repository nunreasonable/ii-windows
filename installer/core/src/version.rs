use std::cmp::Ordering;

use crate::arch::Arch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
	pub parts: Vec<u64>,
	pub pre: Option<String>,
}

impl Version {
	pub fn parse(text: &str) -> Option<Version> {
		let text = text.trim();
		let text = text.strip_prefix('v').or_else(|| text.strip_prefix('V')).unwrap_or(text);
		let (core, pre) = match text.split_once('-') {
			Some((c, p)) => (c, Some(p.to_string())),
			None => (text, None),
		};
		let core = core.split('+').next().unwrap_or(core);
		if core.is_empty() {
			return None;
		}
		let mut parts = Vec::new();
		for piece in core.split('.') {
			parts.push(piece.parse::<u64>().ok()?);
		}
		Some(Version { parts, pre })
	}
}

impl PartialOrd for Version {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Version {
	fn cmp(&self, other: &Self) -> Ordering {
		let len = self.parts.len().max(other.parts.len());
		for i in 0..len {
			let a = self.parts.get(i).copied().unwrap_or(0);
			let b = other.parts.get(i).copied().unwrap_or(0);
			match a.cmp(&b) {
				Ordering::Equal => {}
				o => return o,
			}
		}
		match (&self.pre, &other.pre) {
			(None, None) => Ordering::Equal,
			_ => {
				let (ra, rb) = (pre_rank(self.pre.as_deref()), pre_rank(other.pre.as_deref()));
				if ra != rb {
					return ra.cmp(&rb);
				}
				match (self.pre.as_deref(), other.pre.as_deref()) {
					(Some(a), Some(b)) => match (fix_number(a), fix_number(b)) {
						(Some(x), Some(y)) => x.cmp(&y),
						_ => a.cmp(b),
					},
					_ => Ordering::Equal,
				}
			}
		}
	}
}

fn fix_number(pre: &str) -> Option<u64> {
	pre.to_ascii_lowercase().strip_prefix("fix").and_then(|n| n.parse::<u64>().ok())
}

fn pre_rank(pre: Option<&str>) -> i8 {
	match pre {
		None => 0,
		Some(p) if fix_number(p).is_some() => 1,
		Some(_) => -1,
	}
}

pub fn normalize(text: &str) -> String {
	let t = text.trim();
	t.strip_prefix('v').or_else(|| t.strip_prefix('V')).unwrap_or(t).to_string()
}

pub fn is_newer(candidate: &str, installed: &str) -> bool {
	match (Version::parse(candidate), Version::parse(installed)) {
		(Some(c), Some(i)) => c > i,
		_ => false,
	}
}

pub fn is_prerelease(text: &str) -> bool {
	Version::parse(text).is_some_and(|v| v.pre.as_deref().is_some_and(|p| fix_number(p).is_none()))
}

pub fn same(a: &str, b: &str) -> bool {
	match (Version::parse(a), Version::parse(b)) {
		(Some(x), Some(y)) => x.cmp(&y) == Ordering::Equal,
		_ => normalize(a) == normalize(b),
	}
}

pub fn package_name_parts(name: &str) -> Option<(String, Arch)> {
	let lower = name.to_ascii_lowercase();
	let stem = lower.strip_suffix(".zip")?;
	let rest = stem.strip_prefix("ii-windows-")?;
	let original = &name[name.len() - 4 - rest.len()..name.len() - 4];
	let (ver, arch) = crate::arch::split_package_stem(original);
	Version::parse(ver).map(|_| (normalize(ver), arch))
}

pub fn from_package_name(name: &str) -> Option<String> {
	package_name_parts(name).map(|(v, _)| v)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn fix_releases() {
		assert!(!is_prerelease("0.10.1-fix2"));
		assert!(is_prerelease("1.0.0-beta.3"));
		assert!(is_newer("0.10.1-fix2", "0.10.1"));
		assert!(is_newer("0.10.1-fix3", "0.10.1-fix2"));
		assert!(is_newer("0.10.1-fix10", "0.10.1-fix9"));
		assert!(is_newer("0.10.1-fix2", "0.10.0"));
		assert!(is_newer("1.0.0-beta.3", "0.10.1-fix2"));
		assert!(!is_newer("0.10.1", "0.10.1-fix2"));
	}

	#[test]
	fn year_versions() {
		assert!(is_newer("26.1.1.0", "1.0.8"));
		assert!(is_newer("26.1.1.0", "26.1.1.0-unstable"));
		assert!(is_newer("26.1.1.0-fix1", "26.1.1.0"));
		assert!(is_newer("26.1.1.0-fix2", "26.1.1.0-fix1"));
		assert!(is_newer("26.1.1.1", "26.1.1.0-fix9"));
		assert!(is_newer("26.1.2.0", "26.1.1.3"));
		assert!(is_newer("27.1.2.0", "26.1.1.3-fix4"));
		assert!(is_newer("27.2.0.0", "27.1.9.9"));
		assert!(!is_newer("26.1.1.0", "26.1.1.0"));
		assert!(!is_prerelease("26.1.1.0-fix1"));
		assert!(is_prerelease("26.1.1.0-unstable"));
		assert_eq!(from_package_name("ii-windows-26.1.1.0-fix1.zip").as_deref(), Some("26.1.1.0-fix1"));
	}

	#[test]
	fn ordering() {
		assert!(is_newer("0.2.0", "0.1.0"));
		assert!(is_newer("v0.10.0", "0.9.9"));
		assert!(is_newer("1.0", "0.99.1"));
		assert!(!is_newer("0.1.0", "0.1.0"));
		assert!(!is_newer("0.1.0", "v0.1.0"));
		assert!(is_newer("0.1.0", "0.1.0-rc.1"));
		assert!(!is_newer("0.1.0-rc.1", "0.1.0"));
		assert!(!is_newer("garbage", "0.1.0"));
		assert!(is_prerelease("1.0.0-beta"));
		assert!(!is_prerelease("v0.10.0"));
		assert!(is_newer("1.0.0-beta", "0.10.0"));
		assert!(is_newer("1.0.0", "1.0.0-beta"));
		assert!(same("v0.1.0", "0.1.0"));
		assert!(same("0.1", "0.1.0"));
	}

	#[test]
	fn package_names() {
		assert_eq!(from_package_name("ii-windows-0.1.0.zip").as_deref(), Some("0.1.0"));
		assert_eq!(from_package_name("ii-windows-v1.2.3.zip").as_deref(), Some("1.2.3"));
		assert_eq!(from_package_name("II-Windows-0.3.0-rc.1.ZIP").as_deref(), Some("0.3.0-rc.1"));
		assert_eq!(from_package_name("ii-windows-setup.exe"), None);
		assert_eq!(from_package_name("ii-windows-20261002.zip").as_deref(), Some("20261002"));
		assert_eq!(from_package_name("ii-windows-latest.zip"), None);
		assert_eq!(from_package_name("other-0.1.0.zip"), None);
	}

	#[test]
	fn arm64_package_names() {
		assert_eq!(
			package_name_parts("ii-windows-26.1.1.0-unstable-arm64.zip"),
			Some(("26.1.1.0-unstable".to_string(), Arch::Arm64))
		);
		assert_eq!(
			package_name_parts("ii-windows-26.1.2.0-fix1-arm64.zip"),
			Some(("26.1.2.0-fix1".to_string(), Arch::Arm64))
		);
		assert_eq!(package_name_parts("ii-windows-26.1.2.0.zip"), Some(("26.1.2.0".to_string(), Arch::X64)));
		assert_eq!(from_package_name("ii-windows-26.1.2.0-arm64.zip").as_deref(), Some("26.1.2.0"));
		assert!(!is_prerelease(&from_package_name("ii-windows-26.1.2.0-arm64.zip").unwrap()));
		assert_eq!(package_name_parts("ii-windows-arm64.zip"), None);
		assert_eq!(package_name_parts("ii-windows-26.1.2.0-arm64.zip.sha256"), None);
	}
}
