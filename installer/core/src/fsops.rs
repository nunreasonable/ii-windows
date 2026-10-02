//! File-system helpers: robust delete (antivirus scanners and Explorer hold files for a moment),
//! copy, mirror, and the move-aside swap of a directory's entries with rollback.

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn clear_readonly(path: &Path) {
	if let Ok(meta) = std::fs::symlink_metadata(path) {
		let mut perm = meta.permissions();
		if perm.readonly() {
			#[allow(clippy::permissions_set_readonly_false)]
			perm.set_readonly(false);
			let _ = std::fs::set_permissions(path, perm);
		}
	}
}

fn clear_readonly_tree(path: &Path) {
	clear_readonly(path);
	if let Ok(rd) = std::fs::read_dir(path) {
		for e in rd.flatten() {
			let p = e.path();
			if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
				clear_readonly_tree(&p);
			} else {
				clear_readonly(&p);
			}
		}
	}
}

/// Deletes a file or a whole directory, retrying a few times. Missing is success.
pub fn remove_any(path: &Path) -> io::Result<()> {
	let mut last = None;
	for attempt in 0..6 {
		let meta = match std::fs::symlink_metadata(path) {
			Ok(m) => m,
			Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
			Err(e) => return Err(e),
		};
		let res = if meta.is_dir() { std::fs::remove_dir_all(path) } else { std::fs::remove_file(path) };
		match res {
			Ok(()) => return Ok(()),
			Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
			Err(e) => {
				last = Some(e);
				if attempt == 0 {
					clear_readonly_tree(path);
				}
				std::thread::sleep(Duration::from_millis(150 * (attempt + 1)));
			}
		}
	}
	Err(last.unwrap_or_else(|| io::Error::other("remove failed")))
}

pub fn rename_retry(from: &Path, to: &Path) -> io::Result<()> {
	let mut last = None;
	for attempt in 0..5 {
		match std::fs::rename(from, to) {
			Ok(()) => return Ok(()),
			Err(e) => {
				last = Some(e);
				std::thread::sleep(Duration::from_millis(150 * (attempt + 1)));
			}
		}
	}
	Err(last.unwrap())
}

pub fn copy_dir(src: &Path, dst: &Path) -> io::Result<u64> {
	std::fs::create_dir_all(dst)?;
	let mut n = 0;
	for e in std::fs::read_dir(src)? {
		let e = e?;
		let to = dst.join(e.file_name());
		if e.file_type()?.is_dir() {
			n += copy_dir(&e.path(), &to)?;
		} else {
			std::fs::copy(e.path(), &to)?;
			n += 1;
		}
	}
	Ok(n)
}

/// Makes `dst` an exact copy of `src` in place: copies everything over and deletes what `src`
/// doesn't have, except names in `keep` (matched at any depth, case-insensitively).
pub fn mirror_dir(src: &Path, dst: &Path, keep: &[&str]) -> io::Result<()> {
	std::fs::create_dir_all(dst)?;
	let mut present = HashSet::new();
	for e in std::fs::read_dir(src)? {
		let e = e?;
		present.insert(e.file_name().to_string_lossy().to_lowercase());
		let to = dst.join(e.file_name());
		if e.file_type()?.is_dir() {
			if to.is_file() {
				remove_any(&to)?;
			}
			mirror_dir(&e.path(), &to, keep)?;
		} else {
			if to.is_dir() {
				remove_any(&to)?;
			}
			clear_readonly(&to);
			std::fs::copy(e.path(), &to)?;
		}
	}
	for e in std::fs::read_dir(dst)? {
		let e = e?;
		let name = e.file_name().to_string_lossy().to_lowercase();
		if present.contains(&name) || keep.iter().any(|k| k.eq_ignore_ascii_case(&name)) {
			continue;
		}
		remove_any(&e.path())?;
	}
	Ok(())
}

pub fn dir_size(path: &Path) -> u64 {
	let mut total = 0;
	if let Ok(rd) = std::fs::read_dir(path) {
		for e in rd.flatten() {
			match e.file_type() {
				Ok(t) if t.is_dir() => total += dir_size(&e.path()),
				Ok(_) => total += e.metadata().map(|m| m.len()).unwrap_or(0),
				Err(_) => {}
			}
		}
	}
	total
}

/// Moves a directory's entries aside and the staged ones in, so it can be undone.
///
/// `target` itself is never renamed: a terminal or Explorer window sitting in it would make
/// that fail. Only its entries move, each one a same-volume rename.
pub struct Swap {
	pub target: PathBuf,
	pub previous: PathBuf,
	moved_out: Vec<std::ffi::OsString>,
	moved_in: Vec<std::ffi::OsString>,
	pub staging: PathBuf,
}

impl Swap {
	/// Replaces every entry of `target` with the entries of `staging`, except the `keep` names
	/// already in `target` that `staging` doesn't have (the installer's own files). Old entries
	/// go to `previous` until `commit()` deletes them or `rollback()` puts them back.
	pub fn run(staging: &Path, target: &Path, previous: &Path, keep: &[&str]) -> io::Result<Swap> {
		remove_any(previous)?;
		std::fs::create_dir_all(previous)?;
		std::fs::create_dir_all(target)?;
		let mut swap = Swap {
			target: target.to_path_buf(),
			previous: previous.to_path_buf(),
			moved_out: Vec::new(),
			moved_in: Vec::new(),
			staging: staging.to_path_buf(),
		};
		let incoming: HashSet<String> =
			std::fs::read_dir(staging)?.flatten().map(|e| e.file_name().to_string_lossy().to_lowercase()).collect();
		let result = (|| -> io::Result<()> {
			for e in std::fs::read_dir(target)?.collect::<Vec<_>>() {
				let e = e?;
				let lower = e.file_name().to_string_lossy().to_lowercase();
				if !incoming.contains(&lower) && keep.iter().any(|k| k.eq_ignore_ascii_case(&lower)) {
					continue;
				}
				rename_retry(&e.path(), &previous.join(e.file_name()))
					.map_err(|err| io::Error::new(err.kind(), format!("{} is in use ({err})", e.path().display())))?;
				swap.moved_out.push(e.file_name());
			}
			for e in std::fs::read_dir(staging)?.collect::<Vec<_>>() {
				let e = e?;
				rename_retry(&e.path(), &target.join(e.file_name()))?;
				swap.moved_in.push(e.file_name());
			}
			Ok(())
		})();
		match result {
			Ok(()) => Ok(swap),
			Err(e) => {
				swap.rollback();
				Err(e)
			}
		}
	}

	/// Puts the old entries back (best effort; the caller logs what it can't).
	pub fn rollback(&mut self) -> Vec<String> {
		let mut problems = Vec::new();
		for name in self.moved_in.drain(..).rev() {
			let p = self.target.join(&name);
			if std::fs::rename(&p, self.staging.join(&name)).is_err() {
				if let Err(e) = remove_any(&p) {
					problems.push(format!("{}: {e}", p.display()));
				}
			}
		}
		for name in self.moved_out.drain(..).rev() {
			if let Err(e) = rename_retry(&self.previous.join(&name), &self.target.join(&name)) {
				problems.push(format!("{}: {e}", self.target.join(&name).display()));
			}
		}
		let _ = remove_any(&self.previous);
		problems
	}

	pub fn commit(mut self) -> io::Result<()> {
		self.moved_in.clear();
		self.moved_out.clear();
		let _ = remove_any(&self.staging);
		remove_any(&self.previous)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn tmp(name: &str) -> PathBuf {
		let d = std::env::temp_dir().join(format!("iiw-fs-{}-{name}", std::process::id()));
		let _ = remove_any(&d);
		std::fs::create_dir_all(&d).unwrap();
		d
	}

	fn write(p: &Path, s: &str) {
		std::fs::create_dir_all(p.parent().unwrap()).unwrap();
		std::fs::write(p, s).unwrap();
	}

	#[test]
	fn swap_commit_and_rollback() {
		let d = tmp("swap");
		let target = d.join("ii-windows");
		write(&target.join("qsw.exe"), "old");
		write(&target.join("stale.dll"), "old");
		write(&target.join("install-manifest.json"), "{}");
		write(&target.join("ii-windows-setup.exe"), "old setup");
		let staging = d.join("staging");
		write(&staging.join("qsw.exe"), "new");
		write(&staging.join("config").join("ii").join("shell.qml"), "new");
		write(&staging.join("ii-windows-setup.exe"), "new setup");
		let keep = ["install-manifest.json", "setup.log", "ii-windows-setup.exe"];

		let mut swap = Swap::run(&staging, &target, &d.join("previous"), &keep).unwrap();
		assert_eq!(std::fs::read_to_string(target.join("qsw.exe")).unwrap(), "new");
		assert!(!target.join("stale.dll").exists());
		assert_eq!(std::fs::read_to_string(target.join("install-manifest.json")).unwrap(), "{}");
		assert_eq!(std::fs::read_to_string(target.join("ii-windows-setup.exe")).unwrap(), "new setup");
		let problems = swap.rollback();
		assert!(problems.is_empty(), "{problems:?}");
		assert_eq!(std::fs::read_to_string(target.join("qsw.exe")).unwrap(), "old");
		assert_eq!(std::fs::read_to_string(target.join("stale.dll")).unwrap(), "old");
		assert_eq!(std::fs::read_to_string(target.join("ii-windows-setup.exe")).unwrap(), "old setup");
		assert!(!target.join("config").exists());
		assert!(staging.join("qsw.exe").exists(), "rolled-back entries go back to staging");

		let swap = Swap::run(&staging, &target, &d.join("previous"), &keep).unwrap();
		swap.commit().unwrap();
		assert!(!d.join("previous").exists());
		assert!(!staging.exists());
		assert_eq!(std::fs::read_to_string(target.join("qsw.exe")).unwrap(), "new");
		assert!(target.join("install-manifest.json").exists());
		remove_any(&d).unwrap();
	}

	#[test]
	fn mirror_keeps_listed_names() {
		let d = tmp("mirror");
		let src = d.join("src");
		let dst = d.join("dst");
		write(&src.join("a.qml"), "a2");
		write(&src.join("sub").join("b.qml"), "b2");
		write(&dst.join("a.qml"), "a1");
		write(&dst.join("old.qml"), "x");
		write(&dst.join("sub").join("gone.qml"), "x");
		write(&dst.join("config.json"), "user");
		mirror_dir(&src, &dst, &["config.json"]).unwrap();
		assert_eq!(std::fs::read_to_string(dst.join("a.qml")).unwrap(), "a2");
		assert_eq!(std::fs::read_to_string(dst.join("sub").join("b.qml")).unwrap(), "b2");
		assert!(!dst.join("old.qml").exists());
		assert!(!dst.join("sub").join("gone.qml").exists());
		assert!(dst.join("config.json").exists());
		assert_eq!(dir_size(&src), 4);
		remove_any(&d).unwrap();
	}
}
