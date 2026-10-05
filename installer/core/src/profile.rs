pub const BEGIN: &str = "# >>> illogical-impulse >>>";
pub const END: &str = "# <<< illogical-impulse <<<";

const BLOCK_LINES: [&str; 6] = [
	BEGIN,
	"# Loads illogical-impulse's terminal setup (prompt, colors, aliases). Remove this block, markers",
	"# included, to turn it off; the ii installer removes it on uninstall.",
	"$IiProfile = Join-Path $env:LOCALAPPDATA 'quickshell\\ii\\defaults\\windows\\terminal\\profile.ps1'",
	"if (Test-Path -LiteralPath $IiProfile) { . $IiProfile }",
	END,
];

pub fn block(nl: &str) -> String {
	let mut s = BLOCK_LINES.join(nl);
	s.push_str(nl);
	s
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Encoding {
	Utf16Le,
	Utf16Be,
	Bytes,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EditError {
	Undecodable,
	Unterminated,
}

impl std::fmt::Display for EditError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			EditError::Undecodable => write!(f, "the profile's UTF-16 text could not be decoded"),
			EditError::Unterminated => write!(f, "the profile has a '{BEGIN}' line without a matching '{END}' line"),
		}
	}
}

fn decode(file: &[u8]) -> Result<(Encoding, Vec<u8>), EditError> {
	let utf16 = |be: bool| -> Result<Vec<u8>, EditError> {
		let body = &file[2..];
		if !body.len().is_multiple_of(2) {
			return Err(EditError::Undecodable);
		}
		let units: Vec<u16> = body
			.chunks_exact(2)
			.map(|c| if be { u16::from_be_bytes([c[0], c[1]]) } else { u16::from_le_bytes([c[0], c[1]]) })
			.collect();
		String::from_utf16(&units).map(String::into_bytes).map_err(|_| EditError::Undecodable)
	};
	if file.starts_with(&[0xFF, 0xFE]) {
		Ok((Encoding::Utf16Le, utf16(false)?))
	} else if file.starts_with(&[0xFE, 0xFF]) {
		Ok((Encoding::Utf16Be, utf16(true)?))
	} else {
		Ok((Encoding::Bytes, file.to_vec()))
	}
}

fn encode(enc: Encoding, text: &[u8]) -> Vec<u8> {
	match enc {
		Encoding::Bytes => text.to_vec(),
		Encoding::Utf16Le | Encoding::Utf16Be => {
			let s = String::from_utf8_lossy(text);
			let mut out = if enc == Encoding::Utf16Le { vec![0xFF, 0xFE] } else { vec![0xFE, 0xFF] };
			for unit in s.encode_utf16() {
				let b = if enc == Encoding::Utf16Le { unit.to_le_bytes() } else { unit.to_be_bytes() };
				out.extend_from_slice(&b);
			}
			out
		}
	}
}

fn newline_of(text: &[u8]) -> &'static str {
	if text.windows(2).any(|w| w == b"\r\n") {
		"\r\n"
	} else if text.contains(&b'\n') {
		"\n"
	} else {
		"\r\n"
	}
}

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

fn body(text: &[u8]) -> &[u8] {
	text.strip_prefix(UTF8_BOM).unwrap_or(text)
}

fn lines(text: &[u8]) -> Vec<(usize, usize)> {
	let mut out = Vec::new();
	let mut start = 0;
	for (i, b) in text.iter().enumerate() {
		if *b == b'\n' {
			out.push((start, i + 1));
			start = i + 1;
		}
	}
	if start < text.len() {
		out.push((start, text.len()));
	}
	out
}

fn line_is(text: &[u8], range: (usize, usize), marker: &str) -> bool {
	let line = &text[range.0..range.1];
	let line = body_if_first(line, range.0);
	let trimmed = trim_ascii(line);
	trimmed == marker.as_bytes()
}

fn body_if_first(line: &[u8], start: usize) -> &[u8] {
	if start == 0 {
		body(line)
	} else {
		line
	}
}

fn trim_ascii(s: &[u8]) -> &[u8] {
	let mut a = 0;
	let mut b = s.len();
	while a < b && (s[a] == b' ' || s[a] == b'\t') {
		a += 1;
	}
	while b > a && matches!(s[b - 1], b' ' | b'\t' | b'\r' | b'\n') {
		b -= 1;
	}
	&s[a..b]
}

fn is_blank_line(text: &[u8], range: (usize, usize)) -> bool {
	trim_ascii(body_if_first(&text[range.0..range.1], range.0)).is_empty()
}

pub fn has_block(file: &[u8]) -> bool {
	match decode(file) {
		Ok((_, text)) => lines(&text).into_iter().any(|r| line_is(&text, r, BEGIN)),
		Err(_) => false,
	}
}

pub fn add_block(file: &[u8]) -> Result<Option<Vec<u8>>, EditError> {
	let (enc, mut text) = decode(file)?;
	if lines(&text).into_iter().any(|r| line_is(&text, r, BEGIN)) {
		return Ok(None);
	}
	let nl = newline_of(&text);
	if !trim_ascii(body(&text)).is_empty() {
		if !text.ends_with(b"\n") {
			text.extend_from_slice(nl.as_bytes());
		}
		text.extend_from_slice(nl.as_bytes());
	}
	text.extend_from_slice(block(nl).as_bytes());
	Ok(Some(encode(enc, &text)))
}

pub fn remove_block(file: &[u8]) -> Result<Option<Vec<u8>>, EditError> {
	let (enc, text) = decode(file)?;
	let all = lines(&text);
	let mut remove: Vec<(usize, usize)> = Vec::new();
	let mut i = 0;
	while i < all.len() {
		if line_is(&text, all[i], BEGIN) {
			let end = (i + 1..all.len()).find(|&j| line_is(&text, all[j], END));
			let Some(j) = end else { return Err(EditError::Unterminated) };
			let mut start = all[i].0;
			if i > 0 && is_blank_line(&text, all[i - 1]) && all[i - 1].0 != 0 {
				start = all[i - 1].0;
			}
			let mut stop = all[j].1;
			if all[i].0 == 0 && j + 1 < all.len() && is_blank_line(&text, all[j + 1]) {
				stop = all[j + 1].1;
			}
			remove.push((start, stop));
			i = j + 1;
		} else {
			i += 1;
		}
	}
	if remove.is_empty() {
		return Ok(None);
	}
	let mut out = Vec::with_capacity(text.len());
	let mut pos = 0;
	for (a, b) in remove {
		let a = if a == 0 && text.starts_with(UTF8_BOM) { UTF8_BOM.len() } else { a };
		if a > pos {
			out.extend_from_slice(&text[pos..a]);
		}
		pos = pos.max(b);
	}
	out.extend_from_slice(&text[pos..]);
	Ok(Some(encode(enc, &out)))
}

pub fn is_blank(file: &[u8]) -> bool {
	match decode(file) {
		Ok((_, text)) => trim_all(body(&text)).is_empty(),
		Err(_) => false,
	}
}

fn trim_all(s: &[u8]) -> &[u8] {
	let mut a = 0;
	let mut b = s.len();
	while a < b && s[a].is_ascii_whitespace() {
		a += 1;
	}
	while b > a && s[b - 1].is_ascii_whitespace() {
		b -= 1;
	}
	&s[a..b]
}

#[cfg(test)]
mod tests {
	use super::*;

	fn utf16le(s: &str) -> Vec<u8> {
		let mut v = vec![0xFF, 0xFE];
		for u in s.encode_utf16() {
			v.extend_from_slice(&u.to_le_bytes());
		}
		v
	}

	#[test]
	fn block_matches_the_vm_text() {
		let expected = "# >>> illogical-impulse >>>\r\n\
# Loads illogical-impulse's terminal setup (prompt, colors, aliases). Remove this block, markers\r\n\
# included, to turn it off; the ii installer removes it on uninstall.\r\n\
$IiProfile = Join-Path $env:LOCALAPPDATA 'quickshell\\ii\\defaults\\windows\\terminal\\profile.ps1'\r\n\
if (Test-Path -LiteralPath $IiProfile) { . $IiProfile }\r\n\
# <<< illogical-impulse <<<\r\n";
		assert_eq!(block("\r\n"), expected);
		assert!(block("\n").is_ascii());
	}

	#[test]
	fn new_profile_roundtrip() {
		let added = add_block(b"").unwrap().unwrap();
		assert_eq!(added, block("\r\n").as_bytes());
		assert!(has_block(&added));
		assert_eq!(add_block(&added).unwrap(), None);
		let removed = remove_block(&added).unwrap().unwrap();
		assert!(is_blank(&removed));
		assert!(removed.is_empty());
	}

	#[test]
	fn existing_profile_roundtrip_is_exact() {
		for original in [
			"Set-Alias ll Get-ChildItem\r\nfunction foo { 'bar' }\r\n",
			"Set-Alias ll Get-ChildItem\nno trailing newline handled",
			"Import-Module posh-git\n\n\n",
		] {
			let added = add_block(original.as_bytes()).unwrap().unwrap();
			let text = String::from_utf8(added.clone()).unwrap();
			assert!(text.starts_with(original.trim_end_matches(['\r', '\n']).split('\n').next().unwrap()));
			let removed = remove_block(&added).unwrap().unwrap();
			let removed = String::from_utf8(removed).unwrap();
			if original.ends_with('\n') {
				assert_eq!(removed, original, "roundtrip of {original:?}");
			} else {
				assert_eq!(removed, format!("{original}\n"));
			}
		}
	}

	#[test]
	fn keeps_line_endings_and_non_ascii_bytes() {
		let original: &[u8] = b"Write-Host 'Ol\xe1'\n";
		let added = add_block(original).unwrap().unwrap();
		assert!(added.starts_with(original));
		assert!(!added.windows(2).any(|w| w == b"\r\n"));
		assert_eq!(remove_block(&added).unwrap().unwrap(), original);
	}

	#[test]
	fn utf8_bom_is_kept() {
		let original = b"\xEF\xBB\xBFWrite-Host 'hi'\r\n".to_vec();
		let added = add_block(&original).unwrap().unwrap();
		assert!(added.starts_with(b"\xEF\xBB\xBF"));
		assert_eq!(remove_block(&added).unwrap().unwrap(), original);
		let only_bom = b"\xEF\xBB\xBF".to_vec();
		let added = add_block(&only_bom).unwrap().unwrap();
		let removed = remove_block(&added).unwrap().unwrap();
		assert_eq!(removed, only_bom);
		assert!(is_blank(&removed));
	}

	#[test]
	fn utf16_profiles() {
		let original = utf16le("Write-Host 'ação'\r\n");
		let added = add_block(&original).unwrap().unwrap();
		assert!(added.starts_with(&[0xFF, 0xFE]));
		assert!(has_block(&added));
		assert_eq!(remove_block(&added).unwrap().unwrap(), original);
		assert_eq!(add_block(&[0xFF, 0xFE, 0x41]), Err(EditError::Undecodable));
	}

	#[test]
	fn block_in_the_middle_and_at_the_top() {
		let b = block("\r\n");
		let middle = format!("before\r\n\r\n{b}after\r\n");
		assert_eq!(
			String::from_utf8(remove_block(middle.as_bytes()).unwrap().unwrap()).unwrap(),
			"before\r\nafter\r\n"
		);
		let top = format!("{b}\r\nafter\r\n");
		assert_eq!(String::from_utf8(remove_block(top.as_bytes()).unwrap().unwrap()).unwrap(), "after\r\n");
		assert!(remove_block(b.as_bytes()).unwrap().unwrap().is_empty());
	}

	#[test]
	fn duplicates_and_broken_blocks() {
		let b = block("\n");
		let twice = format!("x\n\n{b}y\n\n{b}");
		assert_eq!(String::from_utf8(remove_block(twice.as_bytes()).unwrap().unwrap()).unwrap(), "x\ny\n");
		let broken = format!("x\n{BEGIN}\nstuff\n");
		assert_eq!(remove_block(broken.as_bytes()), Err(EditError::Unterminated));
		assert_eq!(remove_block(b"nothing here\n").unwrap(), None);
		assert!(!has_block(format!("Write-Host '{BEGIN}'\n").as_bytes()));
	}
}
