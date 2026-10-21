use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
	#[default]
	X64,
	Arm64,
}

pub const MACHINE_X64: u16 = 0x8664;
pub const MACHINE_ARM64: u16 = 0xAA64;

impl Arch {
	pub fn as_str(self) -> &'static str {
		match self {
			Arch::X64 => "x64",
			Arch::Arm64 => "arm64",
		}
	}

	pub fn label(self) -> &'static str {
		match self {
			Arch::X64 => "x64",
			Arch::Arm64 => "ARM64",
		}
	}

	pub fn parse(text: &str) -> Option<Arch> {
		match text.trim().to_ascii_lowercase().as_str() {
			"x64" | "amd64" | "x86_64" => Some(Arch::X64),
			"arm64" | "aarch64" => Some(Arch::Arm64),
			_ => None,
		}
	}

	pub fn from_machine(machine: u16) -> Option<Arch> {
		match machine {
			MACHINE_X64 => Some(Arch::X64),
			MACHINE_ARM64 => Some(Arch::Arm64),
			_ => None,
		}
	}

	pub fn package_suffix(self) -> &'static str {
		match self {
			Arch::X64 => "",
			Arch::Arm64 => "-arm64",
		}
	}

	pub fn package_name(self, version: &str) -> String {
		format!("ii-windows-{version}{}.zip", self.package_suffix())
	}

	pub fn runs_on(self, native: Arch) -> bool {
		self == native || (self == Arch::X64 && native == Arch::Arm64)
	}
}

impl std::fmt::Display for Arch {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}

#[cfg(windows)]
pub fn native() -> Arch {
	crate::win::native_machine().and_then(Arch::from_machine).unwrap_or(Arch::X64)
}

#[cfg(not(windows))]
pub fn native() -> Arch {
	if cfg!(target_arch = "aarch64") {
		Arch::Arm64
	} else {
		Arch::X64
	}
}

pub fn split_package_stem(stem: &str) -> (&str, Arch) {
	let suffix = Arch::Arm64.package_suffix();
	let n = stem.len();
	if n > suffix.len()
		&& stem.is_char_boundary(n - suffix.len())
		&& stem[n - suffix.len()..].eq_ignore_ascii_case(suffix)
	{
		(&stem[..n - suffix.len()], Arch::Arm64)
	} else {
		(stem, Arch::X64)
	}
}

pub fn pe_machine(head: &[u8]) -> Option<u16> {
	if head.len() < 0x40 || &head[..2] != b"MZ" {
		return None;
	}
	let at = u32::from_le_bytes(head[0x3C..0x40].try_into().ok()?) as usize;
	let sig = head.get(at..at + 6)?;
	if &sig[..4] != b"PE\0\0" {
		return None;
	}
	Some(u16::from_le_bytes([sig[4], sig[5]]))
}

pub fn follow(native: Arch, line: Option<Arch>) -> Arch {
	line.filter(|a| a.runs_on(native)).unwrap_or(Arch::X64)
}

pub fn line_after(wanted: Arch, installed: Arch, explicit: bool) -> Arch {
	if !explicit && wanted == Arch::Arm64 && installed == Arch::X64 {
		Arch::Arm64
	} else {
		installed
	}
}

pub fn lenient<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Arch>, D::Error> {
	let v = Option::<serde_json::Value>::deserialize(d)?;
	Ok(v.as_ref().and_then(|v| v.as_str()).and_then(Arch::parse))
}

#[cfg(test)]
mod tests {
	use super::*;

	fn pe(machine: u16) -> Vec<u8> {
		let mut b = vec![0u8; 0x100];
		b[..2].copy_from_slice(b"MZ");
		b[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes());
		b[0x80..0x84].copy_from_slice(b"PE\0\0");
		b[0x84..0x86].copy_from_slice(&machine.to_le_bytes());
		b
	}

	#[test]
	fn names_and_machines() {
		assert_eq!(Arch::X64.package_name("26.1.2.0"), "ii-windows-26.1.2.0.zip");
		assert_eq!(Arch::Arm64.package_name("26.1.2.0"), "ii-windows-26.1.2.0-arm64.zip");
		assert_eq!(split_package_stem("26.1.1.0-unstable-arm64"), ("26.1.1.0-unstable", Arch::Arm64));
		assert_eq!(split_package_stem("26.1.1.0-fix1-ARM64"), ("26.1.1.0-fix1", Arch::Arm64));
		assert_eq!(split_package_stem("26.1.1.0"), ("26.1.1.0", Arch::X64));
		assert_eq!(split_package_stem("-arm64"), ("-arm64", Arch::X64));
		assert_eq!(pe_machine(&pe(MACHINE_ARM64)).and_then(Arch::from_machine), Some(Arch::Arm64));
		assert_eq!(pe_machine(&pe(MACHINE_X64)).and_then(Arch::from_machine), Some(Arch::X64));
		assert_eq!(pe_machine(&pe(0x014C)).and_then(Arch::from_machine), None);
		assert_eq!(pe_machine(b"MZ"), None);
		let mut broken = pe(MACHINE_X64);
		broken[0x3C..0x40].copy_from_slice(&0xFFFFu32.to_le_bytes());
		assert_eq!(pe_machine(&broken), None);
	}

	#[test]
	fn what_runs_where() {
		assert!(Arch::X64.runs_on(Arch::X64));
		assert!(Arch::X64.runs_on(Arch::Arm64));
		assert!(Arch::Arm64.runs_on(Arch::Arm64));
		assert!(!Arch::Arm64.runs_on(Arch::X64));
	}

	#[test]
	fn update_lines() {
		assert_eq!(follow(Arch::Arm64, None), Arch::X64, "an install without a recorded arch is x64");
		assert_eq!(follow(Arch::Arm64, Some(Arch::Arm64)), Arch::Arm64);
		assert_eq!(follow(Arch::Arm64, Some(Arch::X64)), Arch::X64);
		assert_eq!(follow(Arch::X64, Some(Arch::Arm64)), Arch::X64, "an x64 PC never follows arm64");
		assert_eq!(line_after(Arch::Arm64, Arch::X64, false), Arch::Arm64, "a fallback keeps the arm64 line");
		assert_eq!(line_after(Arch::Arm64, Arch::X64, true), Arch::X64, "an explicit x64 package is a choice");
		assert_eq!(line_after(Arch::X64, Arch::Arm64, false), Arch::Arm64);
		assert_eq!(line_after(Arch::X64, Arch::X64, false), Arch::X64);
	}

	#[test]
	fn lenient_values() {
		#[derive(Deserialize)]
		struct O {
			#[serde(default, deserialize_with = "lenient")]
			a: Option<Arch>,
		}
		let p = |s: &str| serde_json::from_str::<O>(s).unwrap().a;
		assert_eq!(p(r#"{"a":"arm64"}"#), Some(Arch::Arm64));
		assert_eq!(p(r#"{"a":"X64"}"#), Some(Arch::X64));
		assert_eq!(p(r#"{"a":"riscv64"}"#), None);
		assert_eq!(p(r#"{"a":7}"#), None);
		assert_eq!(p(r#"{}"#), None);
		assert_eq!(serde_json::to_string(&Arch::Arm64).unwrap(), r#""arm64""#);
	}

	#[test]
	fn native_matches_the_runner() {
		if let Some(want) = std::env::var("IIW_EXPECT_NATIVE_ARCH").ok().and_then(|v| Arch::parse(&v)) {
			assert_eq!(native(), want);
		}
	}
}
