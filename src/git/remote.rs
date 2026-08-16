use crate::error::Result;
use crate::git::repo::Repo;

pub fn fetch(repo: &Repo) -> Result<()> {
    repo.require(&["fetch", "origin"])?;
    Ok(())
}

pub fn push(repo: &Repo, branch: &str, set_upstream: bool) -> Result<()> {
    if set_upstream {
        repo.require(&["push", "-u", "origin", branch])?;
    } else {
        repo.require(&["push"])?;
    }
    Ok(())
}

pub fn push_with_lease(repo: &Repo, branch: &str) -> Result<()> {
    let lease = format!("{branch}:refs/remotes/origin/{branch}");
    let destination = format!("HEAD:refs/heads/{branch}");
    repo.require(&[
        "push",
        &format!("--force-with-lease={lease}"),
        "origin",
        &destination,
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

pub fn remote_head(repo: &Repo, branch: &str) -> Result<Option<String>> {
    let reference = format!("refs/remotes/origin/{branch}");
    let output = repo.run(&["rev-parse", "--verify", "--quiet", &reference])?;
    Ok(output.success.then(|| output.stdout_trimmed().to_string()))
}
