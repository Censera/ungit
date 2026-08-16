use crate::error::{Result, UngitError};
use crate::git::{branch, remote, status};
use crate::output;

/// Starts new work only after the current branch has been brought to the
/// latest shared state.
pub fn run(repo: &crate::git::Repo, name: &str) -> Result<()> {
    if status::is_dirty(repo)? {
        return Err(UngitError::Precondition(
            "there are unsaved changes; save them before starting new work".to_string(),
        ));
    }

    if status::operation_state(repo)? != status::OperationState::Clean {
        return Err(UngitError::Precondition(
            "the repository is in an unfinished Git operation; ungit will not continue from a broken state".to_string(),
        ));
    }

    output::step("Checking for the latest shared work...");
    remote::fetch(repo, None)?;

    let base = branch::default_branch(repo)?.ok_or_else(|| {
        UngitError::Precondition("could not determine the repository's default branch".to_string())
    })?;

    let remote_base = format!("origin/{base}");
    let current = status::current_branch(repo)?.ok_or_else(|| {
        UngitError::Precondition("there is no active branch".to_string())
    })?;
    let relation = status::ahead_behind(repo, &remote_base)?.ok_or_else(|| {
        UngitError::Precondition("could not determine whether the current branch is up to date".to_string())
    })?;

    if current == base && relation.ahead == 0 && relation.behind > 0 {
        output::step("Updating the base branch...");
        repo.require(&["merge", "--ff-only", &remote_base])?;
    } else if current == base && (relation.ahead > 0 || relation.behind > 0) {
        return Err(UngitError::Refused(
            "the base branch contains local work; ungit will not start new work from a divergent base".to_string(),
        ));
    }

    output::step(format!("Starting '{name}' from {remote_base}..."));
    branch::create_and_switch(repo, name, &remote_base)?;
    output::success(format!("Started '{name}'."));
    Ok(())
}
