use crate::error::Result;
use crate::git::repo::Repo;

/// Fetches tracking updates from a specified remote or defaults to origin.
pub fn fetch(repo: &Repo, remote: Option<&str>) -> Result<()> {
    let remote = remote.unwrap_or("origin");
    repo.require(&["fetch", remote])?;
    Ok(())
}

/// Transmits local modifications to the remote repository target.
pub fn push(repo: &Repo, remote: &str, branch: &str, set_upstream: bool) -> Result<()> {
    if set_upstream {
        repo.require(&["push", "-u", remote, branch])?;
    } else {
        repo.require(&["push"])?;
    }
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
}
