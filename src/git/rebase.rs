use crate::error::Result;
use crate::git::repo::Repo;

/// Executes a rebase of the current branch tracking pointer onto a target reference.
pub fn onto(repo: &Repo, onto: &str) -> Result<crate::git::command::GitOutput> {
    repo.run(&["rebase", onto])
}

/// Aborts an in-progress rebase transaction and restores the previous branch tip.
pub fn abort(repo: &Repo) -> Result<()> {
    repo.require(&["rebase", "--abort"])?;
    Ok(())
}

/// Returns file paths containing active conflict markers within the index.
pub fn conflicted_paths(repo: &Repo) -> Result<Vec<String>> {
    let output = repo.require(&["diff", "--name-only", "--diff-filter=U"])?;
    Ok(output.stdout.lines().map(str::to_string).collect())
}
