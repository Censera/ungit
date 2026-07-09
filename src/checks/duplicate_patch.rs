use crate::checks::CheckResult;
use crate::error::Result;
use crate::git::{Repo, commit};
use std::collections::HashMap;

/// Commit count limit for duplicate patch detection.
const SCAN_DEPTH: u32 = 50;

/// Warns if recent history contains duplicate patch IDs.
/// Duplicate IDs indicate mismatched commits with identical content changes.
/// No automatic fix is provided due to ambiguity in manual selection requirements.
pub fn check(repo: &Repo) -> Result<(CheckResult, Option<String>)> {
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
        return Ok((CheckResult::Ok, None));
    }

    let example = &duplicates[0];
    Ok((
        CheckResult::Warning(format!(
            "duplicate patch detected: \"{}\" appears in {} commits ({} total duplicate group(s))",
            example[0].subject,
            example.len(),
            duplicates.len()
        )),
        None,
    ))
}
