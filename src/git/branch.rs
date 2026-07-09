use crate::error::Result;
use crate::git::repo::Repo;

/// Create `name` from `start_point` and check it out in one step
/// (`git switch -c`).
pub fn create_and_switch(repo: &Repo, name: &str, start_point: &str) -> Result<()> {
    repo.require(&["switch", "-c", name, start_point])?;
    Ok(())
}

/// The repository's configured default branch (`main`, `master`, etc.).
///
/// Tries three things in order, each covering a gap in the previous one:
/// 1. The local `refs/remotes/origin/HEAD` symbolic ref, set by `git
///    clone` or `git remote set-head origin -a`.
/// 2. `ls-remote --symref origin HEAD`, which asks the remote directly
///    and doesn't depend on local configuration.
/// 3. Checking whether `main` or `master` actually exists as a branch on
///    `origin`. This covers bare repositories whose own HEAD symref was
///    never repointed (e.g. created with an older git default and never
///    explicitly set), which makes (2) return a branch name that doesn't
///    exist anywhere.
pub fn default_branch(repo: &Repo) -> Result<Option<String>> {
    let local = repo.run(&["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])?;
    if local.success {
        let full = local.stdout_trimmed();
        if let Some(name) = full.strip_prefix("origin/") {
            return Ok(Some(name.to_string()));
        }
    }

    if let Some(name) = remote_symref_head(repo)?
        && remote_branch_exists(repo, &name)? {
            return Ok(Some(name));
    }

    for candidate in ["main", "master"] {
        if remote_branch_exists(repo, candidate)? {
            return Ok(Some(candidate.to_string()));
        }
    }

    Ok(None)
}

/// Asks the remote which ref its own HEAD points to, without checking
/// whether that ref actually exists as a branch.
fn remote_symref_head(repo: &Repo) -> Result<Option<String>> {
    let output = repo.run(&["ls-remote", "--symref", "origin", "HEAD"])?;
    if !output.success {
        return Ok(None);
    }
    // Output looks like: "ref: refs/heads/main\tHEAD\n<hash>\tHEAD"
    for line in output.stdout.lines() {
        if let Some(rest) = line.strip_prefix("ref: refs/heads/")
            && let Some((name, _)) = rest.split_once('\t') {
                return Ok(Some(name.to_string()));
        }
    }
    Ok(None)
}

/// True if `name` exists as a branch on `origin`, checked directly
/// against the remote (not the local `refs/remotes/origin/*` cache).
fn remote_branch_exists(repo: &Repo, name: &str) -> Result<bool> {
    let output = repo.run(&["ls-remote", "--exit-code", "--heads", "origin", name])?;
    Ok(output.success && !output.stdout_trimmed().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::command::test_support::FakeGit;
    use std::path::Path;

    #[test]
    fn default_branch_strips_prefix() {
        let git = FakeGit::new();
        git.push_ok("origin/main\n");
        let repo = Repo {
            root: Path::new("/repo").to_path_buf(),
            executor: &git,
        };
        assert_eq!(default_branch(&repo).unwrap(), Some("main".to_string()));
    }
}
