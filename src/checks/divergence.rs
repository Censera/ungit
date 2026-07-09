use crate::checks::CheckResult;
use crate::error::Result;
use crate::git::{remote, status, Repo};

/// Warns if local and upstream have diverged (both ahead and behind),
/// since that means a rebase or merge will be needed, not a fastforward.
pub fn check(repo: &Repo) -> Result<CheckResult> {
    if remote::upstream_ref(repo)?.is_none() {
        // upstream::check already reports this; avoid double reporting.
        return Ok(CheckResult::Ok);
    }

    let Some(ab) = status::ahead_behind(repo, "@{upstream}")? else {
        return Ok(CheckResult::Ok);
    };

    match (ab.ahead, ab.behind) {
        (0, 0) => Ok(CheckResult::Ok),
        (_, 0) => Ok(CheckResult::Ok), // purely ahead: fine, just needs a push
        (0, _) => Ok(CheckResult::Ok), // purely behind: fine, just needs a pull/rebase
        (ahead, behind) => Ok(CheckResult::Warning(format!(
            "branch has diverged from upstream: {ahead} ahead, {behind} behind"
        ))),
    }
}
