use crate::error::{Result, UngitError};
use crate::git::{branch, remote, status};
use crate::output;

/// Starts a new piece of work from the repository's default branch.
pub fn run(repo: &crate::git::Repo, name: &str) -> Result<()> {
    if status::is_dirty(repo)? {
        return Err(UngitError::Precondition(
            "working tree has unsaved changes; save them before starting new work".to_string(),
        ));
    }

    output::step("Updating repository...");
    remote::fetch(repo, None)?;

    let base = branch::default_branch(repo)?.ok_or_else(|| {
        UngitError::Precondition("could not determine the repository's default branch".to_string())
    })?;

    output::step(format!("Starting '{name}' from {base}..."));
    branch::create_and_switch(repo, name, &format!("origin/{base}"))?;

    output::success(format!("Started '{name}'."));
    Ok(())
}
