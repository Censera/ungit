use crate::error::{Result, UngitError};
use crate::git::{remote, status};
use crate::output;

/// Synchronizes the current work with the shared repository.
///
/// With no local work, sync only brings the branch forward to the fetched
/// remote state. With saved local work, sync reconciles it and publishes it.
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
        if status::ahead_behind(repo, "HEAD")?.is_some() {
            output::info("No remote copy exists yet; nothing to update.");
        }
        return Ok(());
    };

    let ahead = repo.require(&["rev-list", "--count", &format!("{remote_head}..HEAD")])?
        .stdout_trimmed()
        .parse::<u32>()
        .unwrap_or(0);
    let behind = repo.require(&["rev-list", "--count", &format!("HEAD..{remote_head}")])?
        .stdout_trimmed()
        .parse::<u32>()
        .unwrap_or(0);

    if ahead == 0 && behind == 0 {
        output::success("Work is already synced.");
        return Ok(());
    }

    if ahead == 0 && behind > 0 {
        output::step("Updating to the latest shared work...");
        if let Err(error) = repo.require(&["merge", "--ff-only", &remote_head]) {
            let _ = repo.run(&["merge", "--abort"]);
            restore(repo, &original);
            return Err(UngitError::Refused(format!(
                "shared work could not be applied safely; your repository was restored to its previous state. {error}"
            )));
        }
        output::success("Work is up to date.");
        return Ok(());
    }

    output::step("Reconciliating saved work with newer shared work...");
    let result = repo.run(&["rebase", &remote_head])?;
    if !result.success {
        abort_rebase(repo);
        restore(repo, &original);
        return Err(UngitError::Refused(
            "saved work conflicts with newer shared work; your repository was restored to its previous state".to_string(),
        ));
    }

    output::step("Publishing work safely...");
    if let Err(error) = remote::push_with_lease(repo, "origin", &branch) {
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
