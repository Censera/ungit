use crate::checks::CheckResult;
use crate::error::Result;
use crate::git::Repo;
use crate::git::status;

/// Warns if HEAD is not attached to a branch.
/// A detached HEAD is not an error.
/// This state alters `save` and `sync` behavior.
/// No `fix` is offered because branch selection requires user input.
pub fn check(repo: &Repo) -> Result<(CheckResult, Option<String>)> {
    match status::current_branch(repo)? {
        Some(_) => Ok((CheckResult::Ok, None)),
        None => Ok((
            CheckResult::Warning("HEAD is detached (not on any branch)".to_string()),
            None,
        )),
    }
}
