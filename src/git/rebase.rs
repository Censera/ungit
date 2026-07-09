use crate::error::Result;
use crate::git::repo::Repo;

pub fn onto(repo: &Repo, onto: &str) -> Result<crate::git::command::GitOutput> {
    repo.run(&["rebase", onto])
}

/// Abort an in-progress rebase, restoring the branch to its pre-rebase state.
pub fn abort(repo: &Repo) -> Result<()> {
    repo.require(&["rebase", "--abort"])?;
    Ok(())
}

/// Continue an in-progress rebase after conflicts have been resolved and
/// staged.
pub fn continue_(repo: &Repo) -> Result<crate::git::command::GitOutput> {
    repo.run(&["rebase", "--continue"])
}

pub fn conflicted_paths(repo: &Repo) -> Result<Vec<String>> {
    let output = repo.require(&["diff", "--name-only", "--diff-filter=U"])?;
    Ok(output.stdout.lines().map(str::to_string).collect())
}
