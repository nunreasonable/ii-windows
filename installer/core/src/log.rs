use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Log {
	file: Mutex<Option<std::fs::File>>,
	path: Mutex<Option<PathBuf>>,
	sink: Box<dyn Fn(&str) + Send + Sync>,
	pending: Mutex<Vec<String>>,
}

pub fn timestamp() -> String {
	let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
	let secs = now.as_secs() as i64;
	let (y, mo, d, h, mi, s) = civil(secs);
	format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

pub fn stamp() -> String {
	let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
	let (y, mo, d, h, mi, s) = civil(secs);
	format!("{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}")
}

pub fn date_yyyymmdd() -> String {
	stamp()[..8].to_string()
}

fn civil(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
	let days = secs.div_euclid(86400);
	let rem = secs.rem_euclid(86400);
	let z = days + 719468;
	let era = z.div_euclid(146097);
	let doe = z.rem_euclid(146097);
	let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
	let y = yoe + era * 400;
	let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
	let mp = (5 * doy + 2) / 153;
	let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
	let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
	let y = if m <= 2 { y + 1 } else { y };
	(y, m, d, (rem / 3600) as u32, ((rem % 3600) / 60) as u32, (rem % 60) as u32)
}

impl Log {
	pub fn new(sink: Box<dyn Fn(&str) + Send + Sync>) -> Log {
		Log { file: Mutex::new(None), path: Mutex::new(None), sink, pending: Mutex::new(Vec::new()) }
	}

	pub fn open(&self, path: &Path) {
		if let Some(dir) = path.parent() {
			let _ = std::fs::create_dir_all(dir);
		}
		let f = std::fs::OpenOptions::new().create(true).append(true).open(path).ok();
		if let Some(mut f) = f {
			for line in self.pending.lock().unwrap().drain(..) {
				let _ = writeln!(f, "{line}");
			}
			*self.file.lock().unwrap() = Some(f);
			*self.path.lock().unwrap() = Some(path.to_path_buf());
		}
	}

	pub fn close(&self) {
		*self.file.lock().unwrap() = None;
	}

	pub fn path(&self) -> Option<PathBuf> {
		self.path.lock().unwrap().clone()
	}

	pub fn line(&self, text: &str) {
		let line = format!("{} {text}", timestamp());
		match self.file.lock().unwrap().as_mut() {
			Some(f) => {
				let _ = writeln!(f, "{line}");
			}
			None => self.pending.lock().unwrap().push(line.clone()),
		}
		(self.sink)(&line);
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn civil_dates() {
		assert_eq!(super::civil(0), (1970, 1, 1, 0, 0, 0));
		assert_eq!(super::civil(1_790_000_000), (2026, 9, 21, 14, 13, 20));
		assert_eq!(super::civil(951_782_400), (2000, 2, 29, 0, 0, 0));
	}
}
