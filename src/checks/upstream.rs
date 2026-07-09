use crate::checks::CheckResult;
use crate::error::Result;
use crate::git::{Repo, remote, status};

/// Warns if the current branch lacks a configured upstream tracking branch.
pub fn check(repo: &Repo) -> Result<(CheckResult, Option<String>)> {
    if status::current_branch(repo)?.is_none() {
        // Handled by detached_head::check.
        return Ok((CheckResult::Ok, None));
    }

    match remote::upstream_ref(repo)? {
        Some(_) => Ok((CheckResult::Ok, None)),
        None => Ok((
            CheckResult::Warning("current branch has no upstream configured".to_string()),
            Some("ungit sync".to_string()),
        )),
    }
}
