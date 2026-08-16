use crate::error::Result;
use crate::git::repo::Repo;

/// Fetches tracking updates from a specified remote or defaults to origin.
pub fn fetch(repo: &Repo, remote: Option<&str>) -> Result<()> {
    let remote = remote.unwrap_or("origin");
    repo.require(&["fetch", remote])?;
    Ok(())
}

/// Publishes the current branch and optionally creates its upstream relationship.
pub fn push(repo: &Repo, remote: &str, branch: &str, set_upstream: bool) -> Result<()> {
    if set_upstream {
        repo.require(&["push", "-u", remote, branch])?;
    } else {
        repo.require(&["push"])?;
    }
    Ok(())
}

/// Publishes a branch while refusing to replace a remote commit that was not
/// present in the caller's most recent fetch.
pub fn push_with_lease(repo: &Repo, remote: &str, branch: &str) -> Result<()> {
    let lease = format!("{branch}:refs/remotes/{remote}/{branch}");
    repo.require(&[
        "push",
        &format!("--force-with-lease={lease}"),
        remote,
        &format!("HEAD:refs/heads/{branch}"),
    ])?;
    Ok(())
}

/// Resolves the upstream tracking shorthand reference name for the current branch context.
pub fn upstream_ref(repo: &Repo) -> Result<Option<String>> {
    let output = repo.run(&[
        "rev-parse",
        "--abbrev-ref",
        "--symbolic-full-name",
        "@{upstream}",
    ])?;
    if !output.success {
        return Ok(None);
    }
    Ok(Some(output.stdout_trimmed().to_string()))
}

/// Checks whether a fetched remote branch exists.
pub fn remote_branch_exists(repo: &Repo, remote: &str, branch: &str) -> Result<bool> {
    let reference = format!("refs/remotes/{remote}/{branch}");
    let output = repo.run(&["rev-parse", "--verify", "--quiet", &reference])?;
    Ok(output.success)
}

/// Makes the current branch track a remote branch without changing any commits.
pub fn set_upstream(repo: &Repo, remote: &str, branch: &str) -> Result<()> {
    let target = format!("{remote}/{branch}");
    repo.require(&["branch", "--set-upstream-to", &target, branch])?;
    Ok(())
}

pub fn get_url(repo: &Repo, remote: &str) -> Result<Option<String>> {
    let output = repo.run(&["remote", "get-url", remote])?;
    if !output.success {
        return Ok(None);
    }
    Ok(Some(output.stdout_trimmed().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::command::test_support::FakeGit;
    use std::path::Path;

    #[test]
    fn upstream_ref_none_when_unset() {
        let git = FakeGit::new();
        git.push_err("fatal: no upstream configured");
        let repo = Repo {
            root: Path::new("/repo").to_path_buf(),
            executor: &git,
        };
        assert_eq!(upstream_ref(&repo).unwrap(), None);
    }

    #[test]
    fn remote_branch_exists_when_reference_is_present() {
        let git = FakeGit::new();
        git.push_ok("abc123\n");
        let repo = Repo {
            root: Path::new("/repo").to_path_buf(),
            executor: &git,
        };
        assert!(remote_branch_exists(&repo, "origin", "main").unwrap());
    }
}
