//! Persists pre-image state references prior to destructive operations.
//!
//! Handles reader/writer serialization routines for `.git/ungit_journal`
//! while enforcing maximum log retention caps.

use crate::error::{Result, UngitError};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Retention window cap; older log elements are purged on push.
const MAX_ENTRIES: usize = 5;

/// High-level operational categories tracked in journal logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    Sync,
}

impl Operation {
    fn label(self) -> &'static str {
        match self {
            Operation::Sync => "sync",
        }
    }
}

/// Journal entry capturing pre-operation ref targets and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub operation: Operation,
    pub branch: String,
    pub pre_image_sha: String,
    pub recorded_at_unix: u64,
}

impl Entry {
    pub fn describe(&self) -> String {
        format!(
            "{} on '{}', prior to {}",
            self.pre_image_sha,
            self.branch,
            self.operation.label()
        )
    }
}

fn journal_path(git_dir: &Path) -> PathBuf {
    git_dir.join("ungit_journal")
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Records a new state snapshot entry and enforces maximum retention limits.
pub fn record(git_dir: &Path, mut entry: Entry) -> Result<()> {
    entry.recorded_at_unix = now_unix();

    let mut entries = read_all(git_dir)?;
    entries.push(entry);
    if entries.len() > MAX_ENTRIES {
        let drop_count = entries.len() - MAX_ENTRIES;
        entries.drain(0..drop_count);
    }

    write_all(git_dir, &entries)
}

/// Fetches the most recently stored journal entry.
pub fn last(git_dir: &Path) -> Result<Option<Entry>> {
    Ok(read_all(git_dir)?.into_iter().next_back())
}

/// Pops the top-most journal record upon state reversal completion.
pub fn pop_last(git_dir: &Path) -> Result<()> {
    let mut entries = read_all(git_dir)?;
    entries.pop();
    write_all(git_dir, &entries)
}

fn read_all(git_dir: &Path) -> Result<Vec<Entry>> {
    let path = journal_path(git_dir);
    let contents = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(UngitError::Journal(format!("reading {path:?}: {e}"))),
    };

    contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str(line)
                .map_err(|e| UngitError::Journal(format!("parsing entry: {e}")))
        })
        .collect()
}

fn write_all(git_dir: &Path, entries: &[Entry]) -> Result<()> {
    let path = journal_path(git_dir);
    let mut buffer = String::new();
    for entry in entries {
        let line = serde_json::to_string(entry)
            .map_err(|e| UngitError::Journal(format!("serializing entry: {e}")))?;
        buffer.push_str(&line);
        buffer.push('\n');
    }

    let tmp_path = path.with_extension("tmp");
    let mut file = std::fs::File::create(&tmp_path)
        .map_err(|e| UngitError::Journal(format!("creating {tmp_path:?}: {e}")))?;
    file.write_all(buffer.as_bytes())
        .map_err(|e| UngitError::Journal(format!("writing {tmp_path:?}: {e}")))?;
    file.sync_all()
        .map_err(|e| UngitError::Journal(format!("syncing {tmp_path:?}: {e}")))?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|e| UngitError::Journal(format!("renaming {tmp_path:?} to {path:?}: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scratch_dir(label: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "ungit-journal-test-{label}-{}-{}",
            std::process::id(),
            now_unix()
        );
        path.push(unique);
        std::fs::create_dir_all(&path).expect("create scratch dir");
        path
    }

    fn sample_entry(sha: &str) -> Entry {
        Entry {
            operation: Operation::Sync,
            branch: "main".to_string(),
            pre_image_sha: sha.to_string(),
            recorded_at_unix: 0,
        }
    }

    #[test]
    fn last_is_none_when_journal_missing() {
        let dir = scratch_dir("missing");
        assert!(last(&dir).unwrap().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn record_then_last_roundtrips() {
        let dir = scratch_dir("roundtrip");
        record(&dir, sample_entry("abc123")).unwrap();
        let found = last(&dir).unwrap().unwrap();
        assert_eq!(found.pre_image_sha, "abc123");
        assert_eq!(found.branch, "main");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn retention_keeps_only_last_five() {
        let dir = scratch_dir("retention");
        for i in 0..8 {
            record(&dir, sample_entry(&format!("sha{i}"))).unwrap();
        }
        let entries = read_all(&dir).unwrap();
        assert_eq!(entries.len(), MAX_ENTRIES);
        assert_eq!(entries.first().unwrap().pre_image_sha, "sha3");
        assert_eq!(entries.last().unwrap().pre_image_sha, "sha7");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pop_last_removes_most_recent_only() {
        let dir = scratch_dir("pop");
        record(&dir, sample_entry("first")).unwrap();
        record(&dir, sample_entry("second")).unwrap();
        pop_last(&dir).unwrap();
        let found = last(&dir).unwrap().unwrap();
        assert_eq!(found.pre_image_sha, "first");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
