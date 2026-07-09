use crate::checks::CheckResult;
use crate::error::Result;
use crate::git::Repo;
use crate::git::status::OperationState;

/// Errors if a rebase/merge/cherry-pick/etc. is left in progress. This one
/// is `Error`, not `Warning`: an interrupted operation blocks most other
/// commands from working correctly until it's resolved or aborted.
pub fn check(repo: &Repo) -> Result<CheckResult> {
    use crate::git::status::operation_state;

    Ok(match operation_state(repo) {
        OperationState::Clean => CheckResult::Ok,
        OperationState::Rebasing => CheckResult::Error(
            "a rebase is in progress; resolve conflicts or run `ungit repair`".to_string(),
        ),
        OperationState::Merging => CheckResult::Error(
            "a merge is in progress; resolve conflicts or run `ungit repair`".to_string(),
        ),
        OperationState::CherryPicking => CheckResult::Error(
            "a cherry-pick is in progress; resolve conflicts or run `ungit repair`".to_string(),
        ),
        OperationState::Reverting => CheckResult::Error(
            "a revert is in progress; resolve conflicts or run `ungit repair`".to_string(),
        ),
        OperationState::BisectInProgress => {
            CheckResult::Warning("a bisect is in progress".to_string())
        }
    })
}
