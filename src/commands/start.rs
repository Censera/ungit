use crate::error::{Result, UngitError};
use crate::git::{Repo, branch, remote, status};
use crate::output;

/// Fetches remote updates and cuts a new local branch from a base reference tip.
pub fn run(repo: &Repo, name: &str, from: Option<&str>) -> Result<()> {
    if status::is_dirty(repo)? {
        return Err(UngitError::Precondition(
            "working tree has uncommitted changes, save or stash them first".to_string(),
        ));
    }

    output::step("Fetching origin...");
    remote::fetch(repo, None)?;

    let base = match from {
        Some(explicit) => explicit.to_string(),
        None => branch::default_branch(repo)?.ok_or_else(|| {
            UngitError::Precondition(
                "could not determine the default branch, pass --from explicitly".to_string(),
            )
        })?,
    };

    let remote_ref = format!("origin/{base}");
    output::step(format!("Creating '{name}' from {remote_ref}..."));
    branch::create_and_switch(repo, name, &remote_ref)?;

    output::success(format!("Switched to new branch '{name}'."));
    Ok(())
}
