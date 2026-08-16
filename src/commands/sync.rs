use crate::error::{Result, UngitError};
use crate::git::{remote, status};
use crate::output;

/// Synchronizes the current piece of work without exposing Git's intermediate states.
///
/// The operation is transactional from the user's perspective: reconciliation is
/// aborted on conflict, and a failed publication is never allowed to overwrite a
/// collaborator's newer remote work.
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

    let upstream = remote::upstream_ref(repo)?;
    if let Some(upstream) = upstream {
        if status::ahead_behind(repo, &upstream)?.map(|ab| ab.behir).unwrap_or(0) > 0 {
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
    if let Err(error) = remote::push_with_lease(repo, "origin", &branch) {
        restore(repo, &original);
        return Err(UngitError::Refused(format!(
            "publication was refused because the remote changed; your repository was restored. {error}"
        )));
    }

    if remote::upstream_ref(repo)?.is_none() {
        remote::set_upstream(repo, "origin", &branch)?;
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
