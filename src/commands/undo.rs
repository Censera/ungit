use crate::error::{Result, UngitError};
use crate::git::{Repo, commit};
use crate::output;

/// Undoes the last save while keeping its changes in the working tree.
pub fn run(repo: &Repo, hard: bool) -> Result<()> {
    let head = repo.run(&["rev-parse", "--verify", "-q", "HEAD^"])?;
    if !head.success {
        return Err(UngitError::Precondition(
            "nothing to undo: there is no previous save".to_string(),
        ));
    }

    let subject = commit::subject(repo, "HEAD")?;

    if hard {
        output::warning("Discarding the last save and its changes permanently.");
        repo.require(&["reset", "--hard", "HEAD^"])?;
    } else {
        output::step("Undoing the last save...");
        commit::undo_last_soft(repo)?;
    }

    output::success(format!("Undone: {subject}"));
    Ok(())
}
