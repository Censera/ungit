use crate::error::{Result, UngitError};
use crate::git::{Repo, commit};
use crate::output;

/// Reverts the latest commit locally.
/// Performs a soft reset by default to retain uncommitted changes in the index.
/// Discards indexing data and changes fully if `hard` is true.
pub fn run(repo: &Repo, hard: bool) -> Result<()> {
    let head = repo.run(&["rev-parse", "--verify", "-q", "HEAD^"])?;
    if !head.success {
        return Err(UngitError::Precondition(
            "nothing to undo: there is no previous commit".to_string(),
        ));
    }

    let undone_subject = commit::subject(repo, "HEAD")?;

    if hard {
        output::warning("Discarding the last commit and its changes entirely.");
        repo.require(&["reset", "--hard", "HEAD^"])?;
    } else {
        output::step("Undoing last commit, keeping changes in the working tree...");
        commit::undo_last_soft(repo)?;
    }

    output::success(format!("Undone: {undone_subject}"));
    if !hard {
        output::info(
            "Your changes are staged. Adjust and `ungit save` again, or `git reset` to unstage.",
        );
    }
    Ok(())
}
