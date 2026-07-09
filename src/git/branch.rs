use crate::error::Result;
use crate::git::repo::Repo;

/// Create `name` from `start_point` and check it out in one step
/// (`git switch -c`).
pub fn create_and_switch(repo: &Repo, name: &str, start_point: &str) -> Result<()> {
    repo.require(&["switch", "-c", name, start_point])?;
    Ok(())
}

/// Switch to an existing branch.
pub fn switch(repo: &Repo, name: &str) -> Result<()> {
    repo.require(&["switch", name])?;
    Ok(())
}

/// True if `name` exists as a local branch.
pub fn exists(repo: &Repo, name: &str) -> Result<bool> {
    let output = repo.run(&[
        "show-ref",
        "--verify",
        "--quiet",
        &format!("refs/heads/{name}"),
    ])?;
    Ok(output.success)
}

/// The repository's configured default branch (`main`, `master`, etc.).
///
/// Tries the local `refs/remotes/origin/HEAD` symbolic ref first (set by
/// `git clone`, or `git remote set-head origin -a`). That ref is *not*
/// set by a plain `git remote add` + `push -u`, so as a fallback this
/// asks the remote directly via `ls-remote --symref`, which works
/// regardless of how the remote was configured locally.
pub fn default_branch(repo: &Repo) -> Result<Option<String>> {
    let local = repo.run(&["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])?;
    if local.success {
        let full = local.stdout_trimmed();
        if let Some(name) = full.strip_prefix("origin/") {
            return Ok(Some(name.to_string()));
        }
    }

    // Fallback: ask the remote which ref its own HEAD points to.
    let remote_head = repo.run(&["ls-remote", "--symref", "origin", "HEAD"])?;
    if !remote_head.success {
        return Ok(None);
    }
    // Output looks like: "ref: refs/heads/main\tHEAD\n<hash>\tHEAD"
    for line in remote_head.stdout.lines() {
        if let Some(rest) = line.strip_prefix("ref: refs/heads/")
            && let Some((name, _)) = rest.split_once('\t')
        {
            return Ok(Some(name.to_string()));
        }
    }
    Ok(None)
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
