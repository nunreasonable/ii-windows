use serde::Serialize;
use serde_json::{Map, Value};
use std::io;
use std::path::Path;

pub const STYLES: [&str; 2] = ["ii", "end4pc"];
pub const DEFAULT: &str = "ii";
pub const CONFIG_FILE: &str = "config.json";

pub fn is_known(style: &str) -> bool {
	STYLES.contains(&style)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
	Unchanged,
	Created,
	Updated,
}

#[derive(Debug)]
pub enum Error {
	Io(io::Error),
	Parse(String),
	NotObject,
	Unknown(String),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Io(e) => write!(f, "{e}"),
			Error::Parse(e) => write!(f, "not valid JSON ({e})"),
			Error::NotObject => write!(f, "unexpected layout"),
			Error::Unknown(s) => write!(f, "unknown style {s:?}"),
		}
	}
}

fn parse(text: &str) -> Result<Value, Error> {
	serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| Error::Parse(e.to_string()))
}

pub fn read(config: &Path) -> Option<String> {
	let text = std::fs::read_to_string(config).ok()?;
	let root = parse(&text).ok()?;
	let style = root.get("appearance")?.get("visualStyle")?.as_str()?;
	is_known(style).then(|| style.to_string())
}

fn to_text(root: &Value) -> Result<String, Error> {
	let mut out = Vec::new();
	let mut ser = serde_json::Serializer::with_formatter(&mut out, serde_json::ser::PrettyFormatter::with_indent(b"    "));
	root.serialize(&mut ser).map_err(|e| Error::Parse(e.to_string()))?;
	out.push(b'\n');
	String::from_utf8(out).map_err(|e| Error::Parse(e.to_string()))
}

pub fn write(config: &Path, style: &str) -> Result<Outcome, Error> {
	if !is_known(style) {
		return Err(Error::Unknown(style.to_string()));
	}
	let existing = match std::fs::read_to_string(config) {
		Ok(text) => Some(text),
		Err(e) if e.kind() == io::ErrorKind::NotFound => None,
		Err(e) => return Err(Error::Io(e)),
	};
	let mut root = match &existing {
		Some(text) => parse(text)?,
		None if style == DEFAULT => return Ok(Outcome::Unchanged),
		None => Value::Object(Map::new()),
	};
	let appearance = root
		.as_object_mut()
		.ok_or(Error::NotObject)?
		.entry("appearance")
		.or_insert_with(|| Value::Object(Map::new()))
		.as_object_mut()
		.ok_or(Error::NotObject)?;
	let current = appearance.get("visualStyle").and_then(Value::as_str).unwrap_or(DEFAULT);
	if current == style {
		return Ok(Outcome::Unchanged);
	}
	appearance.insert("visualStyle".into(), Value::String(style.into()));
	let text = to_text(&root)?;
	if let Some(dir) = config.parent() {
		std::fs::create_dir_all(dir).map_err(Error::Io)?;
	}
	let tmp = config.with_extension("json.iiw-tmp");
	std::fs::write(&tmp, text).map_err(Error::Io)?;
	if let Err(e) = std::fs::rename(&tmp, config) {
		let _ = std::fs::remove_file(&tmp);
		return Err(Error::Io(e));
	}
	Ok(if existing.is_some() { Outcome::Updated } else { Outcome::Created })
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	fn tmp(name: &str) -> PathBuf {
		let d = std::env::temp_dir().join(format!("iiw-style-{}-{name}", std::process::id()));
		let _ = std::fs::remove_dir_all(&d);
		std::fs::create_dir_all(&d).unwrap();
		d.join(CONFIG_FILE)
	}

	#[test]
	fn missing_config_and_default_style_writes_nothing() {
		let p = tmp("missing-default");
		assert_eq!(write(&p, "ii").unwrap(), Outcome::Unchanged);
		assert!(!p.exists());
		assert_eq!(read(&p), None);
	}

	#[test]
	fn missing_config_gets_only_the_style() {
		let p = tmp("missing-pc");
		assert_eq!(write(&p, "end4pc").unwrap(), Outcome::Created);
		let v: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
		assert_eq!(v, serde_json::json!({"appearance": {"visualStyle": "end4pc"}}));
		assert_eq!(read(&p).as_deref(), Some("end4pc"));
	}

	#[test]
	fn existing_settings_are_kept() {
		let p = tmp("keep");
		let before = serde_json::json!({
			"panelFamily": "ii",
			"appearance": {"extraBackgroundTint": false, "fonts": {"main": "X"}},
			"bar": {"vertical": true, "screenList": ["A", "B"]}
		});
		std::fs::write(&p, serde_json::to_string_pretty(&before).unwrap()).unwrap();
		assert_eq!(write(&p, "end4pc").unwrap(), Outcome::Updated);
		let mut expected = before.clone();
		expected["appearance"]["visualStyle"] = "end4pc".into();
		let after: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
		assert_eq!(after, expected);
		assert!(!p.with_extension("json.iiw-tmp").exists());
	}

	#[test]
	fn same_style_leaves_the_file_alone() {
		let p = tmp("same");
		let text = "{\n  \"appearance\": {\"visualStyle\": \"end4pc\"}, \"x\": 1\n}";
		std::fs::write(&p, text).unwrap();
		assert_eq!(write(&p, "end4pc").unwrap(), Outcome::Unchanged);
		assert_eq!(std::fs::read_to_string(&p).unwrap(), text);
		std::fs::write(&p, "{\"bar\": {}}").unwrap();
		assert_eq!(write(&p, "ii").unwrap(), Outcome::Unchanged);
		assert_eq!(std::fs::read_to_string(&p).unwrap(), "{\"bar\": {}}");
	}

	#[test]
	fn switching_back_to_default_is_written() {
		let p = tmp("back");
		std::fs::write(&p, "{\"appearance\": {\"visualStyle\": \"end4pc\"}}").unwrap();
		assert_eq!(write(&p, "ii").unwrap(), Outcome::Updated);
		assert_eq!(read(&p).as_deref(), Some("ii"));
	}

	#[test]
	fn broken_config_is_not_touched() {
		let p = tmp("broken");
		std::fs::write(&p, "{\"appearance\": ").unwrap();
		assert!(matches!(write(&p, "end4pc"), Err(Error::Parse(_))));
		assert_eq!(std::fs::read_to_string(&p).unwrap(), "{\"appearance\": ");
		std::fs::write(&p, "[1, 2]").unwrap();
		assert!(matches!(write(&p, "end4pc"), Err(Error::NotObject)));
		std::fs::write(&p, "{\"appearance\": 3}").unwrap();
		assert!(matches!(write(&p, "end4pc"), Err(Error::NotObject)));
		assert_eq!(std::fs::read_to_string(&p).unwrap(), "{\"appearance\": 3}");
	}

	#[test]
	fn bom_and_unknown_values() {
		let p = tmp("bom");
		std::fs::write(&p, "\u{feff}{\"appearance\": {\"visualStyle\": \"weird\"}}").unwrap();
		assert_eq!(read(&p), None);
		assert_eq!(write(&p, "end4pc").unwrap(), Outcome::Updated);
		assert_eq!(read(&p).as_deref(), Some("end4pc"));
		assert!(matches!(write(&p, "nope"), Err(Error::Unknown(_))));
	}
}
