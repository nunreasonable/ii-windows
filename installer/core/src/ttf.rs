//! Reads a TrueType/OpenType font's full name (name ID 4), which is what Windows uses for the
//! value name under `...\CurrentVersion\Fonts` ("<full name> (TrueType)").

fn be16(d: &[u8], at: usize) -> Option<u16> {
	Some(u16::from_be_bytes([*d.get(at)?, *d.get(at + 1)?]))
}

fn be32(d: &[u8], at: usize) -> Option<u32> {
	Some(u32::from_be_bytes([*d.get(at)?, *d.get(at + 1)?, *d.get(at + 2)?, *d.get(at + 3)?]))
}

/// Full font name, preferring the Windows/Unicode US-English record.
pub fn full_name(data: &[u8]) -> Option<String> {
	let num_tables = be16(data, 4)? as usize;
	let mut name_off = None;
	for i in 0..num_tables {
		let rec = 12 + i * 16;
		if data.get(rec..rec + 4)? == b"name" {
			name_off = Some(be32(data, rec + 8)? as usize);
			break;
		}
	}
	let t = name_off?;
	let count = be16(data, t + 2)? as usize;
	let storage = t + be16(data, t + 4)? as usize;

	let mut best: Option<(u8, String)> = None;
	for i in 0..count {
		let r = t + 6 + i * 12;
		let platform = be16(data, r)?;
		let encoding = be16(data, r + 2)?;
		let language = be16(data, r + 4)?;
		let name_id = be16(data, r + 6)?;
		let len = be16(data, r + 8)? as usize;
		let off = be16(data, r + 10)? as usize;
		if name_id != 4 {
			continue;
		}
		let raw = data.get(storage + off..storage + off + len)?;
		let (rank, text) = match (platform, encoding) {
			(3, 1) | (3, 10) | (0, _) => {
				let units: Vec<u16> = raw.chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
				let rank = if platform == 3 && language == 0x0409 { 0 } else { 1 };
				(rank, String::from_utf16_lossy(&units))
			}
			(1, 0) => (2, raw.iter().map(|&b| b as char).collect()),
			_ => continue,
		};
		if best.as_ref().is_none_or(|(r, _)| rank < *r) {
			best = Some((rank, text));
		}
	}
	best.map(|(_, s)| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// The registry value name Windows' own font installer would use for this file.
pub fn registry_value_name(data: &[u8], file_name: &str) -> String {
	let base =
		full_name(data).unwrap_or_else(|| file_name.rsplit_once('.').map(|(s, _)| s).unwrap_or(file_name).to_string());
	let otf = file_name.to_ascii_lowercase().ends_with(".otf") && data.starts_with(b"OTTO");
	if otf {
		format!("{base} (OpenType)")
	} else {
		format!("{base} (TrueType)")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reads_jetbrains_nerd_font_if_present() {
		let path = "/usr/share/fonts/jetbrains-mono-nerd-fonts/JetBrainsMonoNerdFont-Regular.ttf";
		if let Ok(data) = std::fs::read(path) {
			let name = full_name(&data).unwrap();
			assert!(name.starts_with("JetBrainsMono"), "{name}");
			assert!(registry_value_name(&data, "JetBrainsMonoNerdFont-Regular.ttf").ends_with(" (TrueType)"));
		}
	}

	#[test]
	fn garbage_falls_back_to_file_name() {
		assert_eq!(full_name(b"not a font"), None);
		assert_eq!(registry_value_name(b"nope", "Foo-Bold.ttf"), "Foo-Bold (TrueType)");
	}
}
