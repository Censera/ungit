use crate::error::{Result, UngitError};
use crate::git::{branch, remote, status};
use crate::output;

/// Starts new work from the latest shared base.
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
    remote::fetch(repo)?;

    let base = branch::default_branch(repo)?.ok_or_else(|| {
        UngitError::Precondition("could not determine the repository's default branch".to_string())
    })?;
    let remote_base = format!("origin/{base}");
    let current = status::current_branch(repo)?
        .ok_or_else(|| UngitError::Precondition("there is no active branch".to_string()))?;
    let relation = status::ahead_behind(repo, &remote_base)?;

    if current == base {
        match (relation.ahead, relation.behind) {
            (0, 0) => {}
            (0, _) => {
                output::step("Updating the base branch...");
                repo.require(&["merge", "--ff-only", &remote_base])?;
            }
            _ => {
                return Err(UngitError::Refused(
                    "the base branch contains local work; ungit will not start new work from a divergent base".to_string(),
                ));
            }
        }
    }

    output::step(format!("Starting '{name}' from {remote_base}..."));
    branch::create_and_switch(repo, name, &remote_base)?;
    output::success(format!("Started '{name}'."));
    Ok(())
}
