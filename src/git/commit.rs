use crate::error::Result;
use crate::git::repo::Repo;

pub fn stage_all(repo: &Repo) -> Result<()> {
    repo.require(&["add", "-A"])?;
    Ok(())
}

pub fn commit(repo: &Repo, message: &str) -> Result<()> {
    repo.require(&["commit", "-m", message])?;
    Ok(())
}

pub fn undo_last_soft(repo: &Repo) -> Result<()> {
    repo.require(&["reset", "--soft", "HEAD^"])?;
    Ok(())
}

pub fn subject(repo: &Repo, rev: &str) -> Result<String> {
    let output = repo.require(&["log", "-1", "--pretty=%s", rev])?;
    Ok(output.stdout_trimmed().to_string())
}
