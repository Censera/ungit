use crate::error::{Result, UngitError};
use crate::git::{Repo, status};
use crate::journal;
use crate::output;

/// Reverts the active branch head to the state recorded prior to the most recent sync.
/// Performs a hard reset to the journaled SHA.
pub fn run(repo: &Repo, confirm: impl Fn(&str) -> bool) -> Result<()> {
    let git_dir = repo.git_dir()?;
    let Some(entry) = journal::last(&git_dir)? else {
        return Err(UngitError::Precondition(
            "no journaled sync to undo".to_string(),
        ));
    };

    let current_branch = status::current_branch(repo)?;
    if current_branch.as_deref() != Some(entry.branch.as_str()) {
        return Err(UngitError::Precondition(format!(
            "last journaled sync was on '{}', but you're on {}, switch back to '{}' first",
            entry.branch,
            current_branch
                .as_deref()
                .map(|b| format!("'{b}'"))
                .unwrap_or_else(|| "a detached HEAD".to_string()),
            entry.branch
        )));
    }

    let prompt = format!(
        "Reset '{}' to {}? This discards the rebase from the last sync.",
        entry.branch, entry.pre_image_sha
    );
    if !confirm(&prompt) {
        return Err(UngitError::Refused("unsync cancelled".to_string()));
    }

    output::step(format!("Resetting to {}...", entry.describe()));
    repo.require(&["reset", "--hard", &entry.pre_image_sha])?;
    journal::pop_last(&git_dir)?;

    output::success(format!(
        "'{}' restored to its pre-sync state.",
        entry.branch
    ));
    output::info("The remote branch was already updated by that sync's push;");
    output::info("you may need `git push --force-with-lease` to reconcile it.");
    Ok(())
}
