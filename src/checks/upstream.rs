use crate::checks::CheckResult;
use crate::error::Result;
use crate::git::{remote, status, Repo};

/// Warns if the current branch has no upstream configured. Not an error:
/// a brand new local branch legitimately has none yet, and `sync`/`start`
/// know how to set one automatically.
pub fn check(repo: &Repo) -> Result<CheckResult> {
    // Detached HEAD has no "current branch" concept for upstreams, that's
    // detached_head's concern, not this one's
    if status::current_branch(repo)?.is_none() {
        return Ok(CheckResult::Ok);
    }

    match remote::upstream_ref(repo)? {
        Some(_) => Ok(CheckResult::Ok),
        None => Ok(CheckResult::Warning(
            "current branch has no upstream configured".to_string(),
        )),
    }
}
