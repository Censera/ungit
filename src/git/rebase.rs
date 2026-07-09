use crate::error::Result;
use crate::git::repo::Repo;

/// Rebase the current branch onto `onto` (e.g. `"origin/main"` or
/// `"@{upstream}"`). Does not resolve conflicts; a conflicted rebase
/// returns `Ok` with `success: false` in the underlying output, which the
/// caller (`commands::sync`) inspects via `Repo::run` semantics — so this
/// wrapper uses `run`, not `require`, deliberately: a conflict is an
/// expected outcome, not a plumbing failure.
pub fn onto(repo: &Repo, onto: &str) -> Result<crate::git::command::GitOutput> {
    repo.run(&["rebase", onto])
}

pub fn abort(repo: &Repo) -> Result<()> {
    repo.require(&["rebase", "--abort"])?;
    Ok(())
}

/// Paths currently marked as conflicted (unmerged) in the index.
pub fn conflicted_paths(repo: &Repo) -> Result<Vec<String>> {
    let output = repo.require(&["diff", "--name-only", "--diff-filter=U"])?;
    Ok(output.stdout.lines().map(str::to_string).collect())
}
