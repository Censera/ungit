use crate::checks::CheckResult;
use crate::error::Result;
use crate::git::{Repo, remote, status};

/// Warns if local and upstream branches have diverged.
/// Diverged branches require a merge or rebase.
pub fn check(repo: &Repo) -> Result<(CheckResult, Option<String>)> {
    if remote::upstream_ref(repo)?.is_none() {
        // Handled by upstream::check to prevent duplicate warnings.
        return Ok((CheckResult::Ok, None));
    }

    let Some(ab) = status::ahead_behind(repo, "@{upstream}")? else {
        return Ok((CheckResult::Ok, None));
    };

    match (ab.ahead, ab.behind) {
        (0, 0) => Ok((CheckResult::Ok, None)),
        (_, 0) => Ok((CheckResult::Ok, None)), // Ahead only. Fast-forward push possible.
        (0, _) => Ok((CheckResult::Ok, None)), // Behind only. Fast-forward pull possible.
        (ahead, behind) => Ok((
            CheckResult::Warning(format!(
                "branch has diverged from upstream: {ahead} ahead, {behind} behind"
            )),
            Some("ungit sync".to_string()),
        )),
    }
}
