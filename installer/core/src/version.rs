use std::cmp::Ordering;

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
			(None, Some(_)) => Ordering::Greater,
			(Some(_), None) => Ordering::Less,
			(Some(a), Some(b)) => a.cmp(b),
		}
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

pub fn same(a: &str, b: &str) -> bool {
	match (Version::parse(a), Version::parse(b)) {
		(Some(x), Some(y)) => x.cmp(&y) == Ordering::Equal,
		_ => normalize(a) == normalize(b),
	}
}

pub fn from_package_name(name: &str) -> Option<String> {
	let lower = name.to_ascii_lowercase();
	let stem = lower.strip_suffix(".zip")?;
	let rest = stem.strip_prefix("ii-windows-")?;
	let original = &name[name.len() - 4 - rest.len()..name.len() - 4];
	Version::parse(original).map(|_| normalize(original))
}

#[cfg(test)]
mod tests {
	use super::*;

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
}
