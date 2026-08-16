use crate::error::{Result, UngitError};
use crate::git::{remote, status};
use crate::output;

/// Publishes saved work after reconciling it with newer shared work.
pub fn run(repo: &crate::git::Repo) -> Result<()> {
    let branch = status::current_branch(repo)?
        .ok_or_else(|| UngitError::Precondition("there is no active piece of work".to_string()))?;

    if status::operation_state(repo)? != status::OperationState::Clean {
        return Err(UngitError::Precondition(
            "the repository is in an unfinished operation; ungit will not continue from a broken state".to_string(),
        ));
    }

    if status::is_dirty(repo)? {
        return Err(UngitError::Precondition(
            "there are unsaved changes; save them before publishing".to_string(),
        ));
    }

    let original = repo.require(&["rev-parse", "HEAD"])?.stdout_trimmed().to_string();

    output::step("Checking for new shared work...");
    remote::fetch(repo)?;

    let Some(shared) = remote::remote_head(repo, &branch)? else {
        output::step("Publishing work for the first time...");
        remote::push(repo, &branch, true)?;
        output::success("Work is published.");
        return Ok(());
    };

    let relation = status::ahead_behind(repo, &shared)?;
    match (relation.ahead, relation.behind) {
        (0, 0) => output::success("Work is already published."),
        (0, _) => match repo.require(&["merge", "--ff-only", &shared]) {
            Ok(_) => output::success("Work is up to date."),
            Err(error) => return Err(rollback(repo, &original, format!("shared work could not be applied safely: {error}"))),
        },
        _ => {
            output::step("Reconciling saved work with newer shared work...");
            let result = repo.run(&["rebase", &shared])?;
            if !result.success {
                let reason = if result.stderr.trim().is_empty() {
                    "saved work conflicts with newer shared work".to_string()
                } else {
                    format!("saved work could not be reconciled: {}", result.stderr.trim())
                };
                return Err(abort_and_rollback(repo, &original, reason));
            }

            output::step("Publishing work safely...");
            if let Err(error) = remote::push_with_lease(repo, &branch) {
                return Err(rollback(repo, &original, format!("publication was refused: {error}")));
            }
            output::success("Work is published.");
        }
    }

    Ok(())
}

fn abort_and_rollback(repo: &crate::git::Repo, original: &str, reason: String) -> UngitError {
    match repo.require(&["rebase", "--abort"]) {
        Ok(_) => rollback(repo, original, reason),
        Err(error) => UngitError::Refused(format!("{reason}; aborting reconciliation also failed: {error}")),
    }
}

fn rollback(repo: &crate::git::Repo, original: &str, reason: String) -> UngitError {
    match repo.require(&["reset", "--hard", original]) {
        Ok(_) => UngitError::Refused(format!("{reason}; your repository was restored to its previous state")),
        Err(error) => UngitError::Refused(format!("{reason}; restoring the previous repository state also failed: {error}")),
    }
}
