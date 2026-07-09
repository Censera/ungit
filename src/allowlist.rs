//! Persists which `checks::run_all` findings the user has silenced, by
//! `Finding::name`.
//!
//! This module owns one concern: reading and writing
//! `.git/ungit_allow`. It does not decide *which* findings exist (that's
//! `checks::run_all`) and it does not decide *how* `check`/`repair`
//! behave toward a silenced finding (that's `commands::check` and
//! `commands::repair`). It only knows the file format: one finding name
//! per line, unordered, no duplicates.

use crate::error::{Result, UngitError};
use std::path::{Path, PathBuf};

fn allowlist_path(git_dir: &Path) -> PathBuf {
    git_dir.join("ungit_allow")
}

/// The full set of currently-silenced finding names.
pub fn read(git_dir: &Path) -> Result<Vec<String>> {
    let path = allowlist_path(git_dir);
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(UngitError::Allowlist(format!("reading {path:?}: {e}"))),
    }
}

/// Add `name` to the allowlist. No-op if already present.
pub fn add(git_dir: &Path, name: &str) -> Result<()> {
    let mut names = read(git_dir)?;
    if names.iter().any(|n| n == name) {
        return Ok(());
    }
    names.push(name.to_string());
    write(git_dir, &names)
}

/// Remove `name` from the allowlist. No-op if not present.
pub fn remove(git_dir: &Path, name: &str) -> Result<()> {
    let mut names = read(git_dir)?;
    names.retain(|n| n != name);
    write(git_dir, &names)
}

fn write(git_dir: &Path, names: &[String]) -> Result<()> {
    use std::io::Write;

    let path = allowlist_path(git_dir);
    let mut buffer = String::new();
    for name in names {
        buffer.push_str(name);
        buffer.push('\n');
    }

    // Temp file plus rename, same as journal.rs: a crash mid-write can't
    // corrupt the file or leave a half-written line for read() to choke on.
    let tmp_path = path.with_extension("tmp");
    let mut file = std::fs::File::create(&tmp_path)
        .map_err(|e| UngitError::Allowlist(format!("creating {tmp_path:?}: {e}")))?;
    file.write_all(buffer.as_bytes())
        .map_err(|e| UngitError::Allowlist(format!("writing {tmp_path:?}: {e}")))?;
    file.sync_all()
        .map_err(|e| UngitError::Allowlist(format!("syncing {tmp_path:?}: {e}")))?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|e| UngitError::Allowlist(format!("renaming {tmp_path:?} to {path:?}: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch `.git`-like directory for one test. Same pattern as
    /// `journal.rs`'s test scaffolding: cleanup is best-effort via an
    /// explicit call at the end of each test, not a Drop impl.
    fn scratch_dir(label: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "ungit-allowlist-test-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        path.push(unique);
        std::fs::create_dir_all(&path).expect("create scratch dir");
        path
    }

    #[test]
    fn read_is_empty_when_file_missing() {
        let dir = scratch_dir("missing");
        assert_eq!(read(&dir).unwrap(), Vec::<String>::new());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn add_then_present_in_read() {
        let dir = scratch_dir("add");
        add(&dir, "ignored-files").unwrap();
        let names = read(&dir).unwrap();
        assert!(names.iter().any(|n| n == "ignored-files"));
        assert!(!names.iter().any(|n| n == "merge-state"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn add_is_idempotent() {
        let dir = scratch_dir("idempotent");
        add(&dir, "ignored-files").unwrap();
        add(&dir, "ignored-files").unwrap();
        assert_eq!(read(&dir).unwrap(), vec!["ignored-files".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn remove_clears_only_named_entry() {
        let dir = scratch_dir("remove");
        add(&dir, "ignored-files").unwrap();
        add(&dir, "merge-state").unwrap();
        remove(&dir, "ignored-files").unwrap();
        let names = read(&dir).unwrap();
        assert!(!names.iter().any(|n| n == "ignored-files"));
        assert!(names.iter().any(|n| n == "merge-state"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn remove_of_absent_entry_is_a_no_op() {
        let dir = scratch_dir("remove-absent");
        add(&dir, "merge-state").unwrap();
        remove(&dir, "does-not-exist").unwrap();
        assert_eq!(read(&dir).unwrap(), vec!["merge-state".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
