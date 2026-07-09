use crate::checks::CheckResult;
use crate::error::Result;
use crate::git::{Repo, commit};
use std::collections::HashMap;

/// How many recent commits to scan for duplicate patches. Kept small: this
/// is a cheap sanity check, not a full history audit.
const SCAN_DEPTH: u32 = 50;

/// Warns if two commits in recent history introduce the same patch (same
/// `git patch-id`), which usually means a cherrypick or rebase went
/// sideways and duplicated a change under a different commit hash.
pub fn check(repo: &Repo) -> Result<CheckResult> {
    let patches = commit::recent_patch_ids(repo, SCAN_DEPTH)?;

    let mut by_patch_id: HashMap<&str, Vec<&commit::CommitPatch>> = HashMap::new();
    for p in &patches {
        if p.patch_id.is_empty() {
            continue;
        }
        by_patch_id.entry(&p.patch_id).or_default().push(p);
    }

    let duplicates: Vec<_> = by_patch_id.values().filter(|v| v.len() > 1).collect();

    if duplicates.is_empty() {
        return Ok(CheckResult::Ok);
    }

    let example = &duplicates[0];
    Ok(CheckResult::Warning(format!(
        "duplicate patch detected: \"{}\" appears in {} commits ({} total duplicate group(s))",
        example[0].subject,
        example.len(),
        duplicates.len()
    )))
}
