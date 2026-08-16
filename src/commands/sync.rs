use crate::error::{Result, UngitError};
use crate::git::{remote, status};
use crate::output;

/// Reconciles local work with the remote and publishes it.
/// Git's rebase and upstream state are implementation details of this action.
pub fn run(repo: &crate::git::Repo) -> Result<()> {
    let branch = status::current_branch(repo)?.ok_or_else(|| {
        UngitError::Precondition("repository is not in a normal working state".to_string())
    })?;

    if status::is_dirty(repo)? {
        return Err(UngitError::Precondition(
            "working tree has unsaved changes; save them before syncing".to_string(),
        ));
    }

    output::step("Updating repository...");
    remote::fetch(repo, None)?;

    if let Some(upstream) = remote::upstream_ref(repo)? {
        output::step("Reconciling local work...");
        let result = repo.run(&["rebase", &upstream])?;
        if !result.success {
            let _ = repo.run(&["rebase", "--abort"])?;
            return Err(UngitError::Refused(
                "sync could not reconcile the changes; nothing was left half-finished".to_string(),
            ));
        }
        remote::push(repo, "origin", &branch, false)?;
    } else {
        output::step("Publishing work...");
        remote::push(repo, "origin", &branch, true)?;
    }

    output::success("Work is synced.");
    Ok(())
}
