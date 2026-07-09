use crate::checks::CheckResult;
use crate::error::Result;
use crate::git::status;
use crate::git::Repo;

/// Warns if HEAD is not attached to a branch. Detached HEAD is not an
/// error by itself (checking out a tag/commit is normal), but it's worth
/// surfacing since `save`/`sync` behave differently there.
pub fn check(repo: &Repo) -> Result<CheckResult> {
    match status::current_branch(repo)? {
        Some(_) => Ok(CheckResult::Ok),
        None => Ok(CheckResult::Warning(
            "HEAD is detached (not on any branch)".to_string(),
        )),
    }
}
