use crate::error::Result;
use crate::git::repo::Repo;

pub fn fetch(repo: &Repo, remote: Option<&str>) -> Result<()> {
    repo.require(&["fetch", remote.unwrap_or("origin")])?;
    Ok(())
}

pub fn push(repo: &Repo, remote: &str, branch: &str, set_upstream: bool) -> Result<()> {
    if set_upstream {
        repo.require(&["push", "-u", remote, branch])?;
    } else {
        repo.require(&["push"])?;
    }
    Ok(())
}

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

pub fn upstream_ref(repo: &Repo) -> Result<Option<String>> {
    let output = repo.run(&[
        "rev-parse",
        "--abbrev-ref",
        "--symbolic-full-name",
        "@{upstream}",
    ])?;
    Ok(output.success.then(|| output.stdout_trimmed().to_string()))
}

pub fn remote_head(repo: &Repo, remote: &str, branch: &str) -> Result<Option<String>> {
    let reference = format!("refs/remotes/{remote}/{branch}");
    let output = repo.run(&["rev-parse", "--verify", "--quiet", &reference])?;
    Ok(output.success.then(|| output.stdout_trimmed().to_string()))
}

pub fn remote_branch_exists(repo: &Repo, remote: &str, branch: &str) -> Result<bool> {
    Ok(remote_head(repo, remote, branch)?.is_some())
}

pub fn set_upstream(repo: &Repo, remote: &str, branch: &str) -> Result<()> {
    let target = format!("{remote}/{branch}");
    repo.require(&["branch", "--set-upstream-to", &target, branch])?;
    Ok(())
}

pub fn get_url(repo: &Repo, remote: &str) -> Result<Option<String>> {
    let output = repo.run(&["remote", "get-url", remote])?;
    Ok(output.success.then(|| output.stdout_trimmed().to_string()))
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
        let repo = Repo { root: Path::new("/repo").to_path_buf(), executor: &git };
        assert_eq!(upstream_ref(&repo).unwrap(), None);
    }

    #[test]
    fn remote_head_reads_fetched_reference() {
        let git = FakeGit::new();
        git.push_ok("abc123\n");
        let repo = Repo { root: Path::new("/repo").to_path_buf(), executor: &git };
        assert_eq!(remote_head(&repo, "origin", "main").unwrap(), Some("abc123".into()));
    }
}
