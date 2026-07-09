use crate::error::Result;
use crate::git::repo::Repo;

/// Creates a new branch from a starting point and switches to it.
pub fn create_and_switch(repo: &Repo, name: &str, start_point: &str) -> Result<()> {
    repo.require(&["switch", "-c", name, start_point])?;
    Ok(())
}

/// Resolves the configured default branch name of the remote repository.
///
/// Checks local tracking reference targets, remote symref configurations,
/// and fallback branch configurations sequentially.
pub fn default_branch(repo: &Repo) -> Result<Option<String>> {
    let local = repo.run(&["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])?;
    if local.success {
        let full = local.stdout_trimmed();
        if let Some(name) = full.strip_prefix("origin/") {
            return Ok(Some(name.to_string()));
        }
    }

    if let Some(name) = remote_symref_head(repo)?
        && remote_branch_exists(repo, &name)?
    {
        return Ok(Some(name));
    }

    for candidate in ["main", "master"] {
        if remote_branch_exists(repo, candidate)? {
            return Ok(Some(candidate.to_string()));
        }
    }

    Ok(None)
}

/// Identifies the reference targeted by the remote HEAD symbolic reference.
fn remote_symref_head(repo: &Repo) -> Result<Option<String>> {
    let output = repo.run(&["ls-remote", "--symref", "origin", "HEAD"])?;
    if !output.success {
        return Ok(None);
    }
    for line in output.stdout.lines() {
        if let Some(rest) = line.strip_prefix("ref: refs/heads/")
            && let Some((name, _)) = rest.split_once('\t')
        {
            return Ok(Some(name.to_string()));
        }
    }
    Ok(None)
}

/// Evaluates whether a branch exists on the remote target.
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
