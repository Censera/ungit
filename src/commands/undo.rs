use crate::error::{Result, UngitError};
use crate::git::{Repo, commit};
use crate::output;

/// `ungit undo [--hard]`
///
/// Undo the last commit. By default this is a soft reset: the commit is
/// gone but its changes remain staged in the working tree, so nothing is
/// destroyed. `--hard` additionally discards those changes that is a
/// real deletion and the caller (`main`) is expected to have confirmed
/// with the user before reaching here.
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
