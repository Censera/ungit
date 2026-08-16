use crate::error::{Result, UngitError};
use crate::git::{remote, status};
use crate::output;

/// Synchronizes the current piece of work without exposing Git's intermediate states.
///
/// Reconciliation is aborted on conflict, and publication is protected against
/// collaborators changing the remote after the fetch.
pub fn run(repo: &crate::git::Repo) -> Result<()> {
    let branch = status::current_branch(repo)?.ok_or_else(|| {
        UngitError::Precondition("there is no active piece of work".to_string())
    })?;

    if status::is_dirty(repo)? {
        return Err(UngitError::Precondition(
            "there are unsaved changes; save them before syncing".to_string(),
        ));
    }

    if status::operation_state(repo)? != status::OperationState::Clean {
        return Err(UngitError::Precondition(
            "the repository is in an unfinished Git operation; ungit will not continue from a broken state".to_string(),
        ));
    }

    let original = repo.require(&["rev-parse", "HEAD"])?.stdout_trimmed().to_string();

    output::step("Checking for new work...");
    remote::fetch(repo, None)?;

    let has_remote_branch = remote::remote_branch_exists(repo, "origin", &branch)?;

    if has_remote_branch {
        let upstream = remote::upstream_ref(repo)?.unwrap_or_else(|| format!("origin/{branch}"));
        let ahead_behind = status::ahead_behind(repo, &upstream)?.ok_or_else(|| {
            UngitError::Refused("could not determine the relationship with remote work".to_string())
        })?;

        if ahead_behind.behind > 0 {
            output::step("Reconciliating with newer work...");
            let result = repo.run(&["rebase", &upstream])?;
            if !result.success {
                abort_rebase(repo);
                restore(repo, &original);
                return Err(UngitError::Refused(
                    "the work could not be reconciled cleanly; your repository was restored to its previous state".to_string(),
                ));
            }
        }
    }

    output::step("Publishing work safely...");
    let push_result = if has_remote_branch {
        remote::push_with_lease(repo, "origin", &branch)
    } else {
        remote::push(repo, "origin", &branch, true)
    };

    if let Err(error) = push_result {
        restore(repo, &original);
        return Err(UngitError::Refused(format!(
            "publication was refused; your repository was restored to its previous state. {error}"
        )));
    }

    output::success("Work is synced.");
    Ok(())
}

fn abort_rebase(repo: &crate::git::Repo) {
    let _ = repo.run(&["rebase", "--abort"]);
}

fn restore(repo: &crate::git::Repo, original: &str) {
    let _ = repo.run(&["reset", "--hard", original]);
}
