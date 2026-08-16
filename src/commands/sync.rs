use crate::error::{Result, UngitError};
use crate::git::{remote, status};
use crate::output;

/// Synchronizes the current work with the shared repository.
///
/// The repository starts clean, reconciliation is transactional, and a failed
/// operation is reported with its rollback result instead of being swallowed.
pub fn run(repo: &crate::git::Repo) -> Result<()> {
    let branch = status::current_branch(repo)?.ok_or_else(|| {
        UngitError::Precondition("there is no active piece of work".to_string())
    })?;

    if status::operation_state(repo)? != status::OperationState::Clean {
        return Err(UngitError::Precondition(
            "the repository is in an unfinished Git operation; ungit will not continue from a broken state".to_string(),
        ));
    }

    if status::is_dirty(repo)? {
        return Err(UngitError::Precondition(
            "there are unsaved changes; save them before syncing".to_string(),
        ));
    }

    let original = repo.require(&["rev-parse", "HEAD"])?.stdout_trimmed().to_string();

    output::step("Checking for new work...");
    remote::fetch(repo, None)?;

    let Some(remote_head) = remote::remote_head(repo, "origin", &branch)? else {
        output::step("Publishing work for the first time...");
        remote::push(repo, "origin", &branch, true)?;
        output::success("Work is synced.");
        return Ok(());
    };

    let ahead = commit_count(repo, &format!("{remote_head}..HEAD"))?;
    let behind = commit_count(repo, &format!("HEAD..{remote_head}"))?;

    match (ahead, behind) {
        (0, 0) => {
            output::success("Work is already synced.");
            Ok(())
        }
        (0, _) => {
            output::step("Updating to the latest shared work...");
            match repo.require(&["merge", "--ff-only", &remote_head]) {
                Ok(_) => {
                    output::success("Work is up to date.");
                    Ok(())
                }
                Err(error) => Err(rollback_error(
                    repo,
                    &original,
                    format!("shared work could not be applied safely: {error}"),
                )),
            }
        }
        _ => {
            output::step("Reconciling saved work with newer shared work...");
            let result = repo.run(&["rebase", &remote_head])?;
            if !result.success {
                let rebase_error = result.stderr.trim();
                let message = if rebase_error.is_empty() {
                    "saved work conflicts with newer shared work".to_string()
                } else {
                    format!("saved work could not be reconciled: {rebase_error}")
                };
                return Err(rollback_after_rebase(repo, &original, message));
            }

            output::step("Publishing work safely...");
            if let Err(error) = remote::push_with_lease(repo, "origin", &branch) {
                return Err(rollback_error(
                    repo,
                    &original,
                    format!("publication was refused: {error}"),
                ));
            }

            output::success("Work is synced.");
            Ok(())
        }
    }
}

fn commit_count(repo: &crate::git::Repo, range: &str) -> Result<u32> {
    let output = repo.require(&["rev-list", "--count", range])?;
    output
        .stdout_trimmed()
        .parse::<u32>()
        .map_err(|error| UngitError::Precondition(format!("invalid commit count from Git: {error}")))
}

fn rollback_after_rebase(repo: &crate::git::Repo, original: &str, reason: String) -> UngitError {
    match repo.require(&["rebase", "--abort"]) {
        Ok(_) => rollback_error(repo, original, reason),
        Err(error) => UngitError::Refused(format!(
            "{reason}; rebase abort also failed: {error}"
        )),
    }
}

fn rollback_error(repo: &crate::git::Repo, original: &str, reason: String) -> UngitError {
    match repo.require(&["reset", "--hard", original]) {
        Ok(_) => UngitError::Refused(format!(
            "{reason}; your repository was restored to its previous state"
        )),
        Err(error) => UngitError::Refused(format!(
            "{reason}; restoring the previous repository state also failed: {error}"
        )),
    }
}
