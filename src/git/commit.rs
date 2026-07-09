use crate::error::Result;
use crate::git::repo::Repo;

/// Stage every tracked and untracked path (`git add -A`).
pub fn stage_all(repo: &Repo) -> Result<()> {
    repo.require(&["add", "-A"])?;
    Ok(())
}

/// Stage specific paths.
pub fn stage(repo: &Repo, paths: &[&str]) -> Result<()> {
    let mut args = vec!["add", "--"];
    args.extend(paths);
    repo.require(&args)?;
    Ok(())
}

/// Create a commit with the given message from whatever is currently staged.
pub fn commit(repo: &Repo, message: &str) -> Result<()> {
    repo.require(&["commit", "-m", message])?;
    Ok(())
}

/// Undo the last commit, keeping its changes staged in the working tree
/// (`git reset --soft HEAD~1` semantics via `HEAD^`).
pub fn undo_last_soft(repo: &Repo) -> Result<()> {
    repo.require(&["reset", "--soft", "HEAD^"])?;
    Ok(())
}

pub fn subject(repo: &Repo, rev: &str) -> Result<String> {
    let output = repo.require(&["log", "-1", "--pretty=%s", rev])?;
    Ok(output.stdout_trimmed().to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitPatch {
    pub hash: String,
    pub patch_id: String,
    pub subject: String,
}

pub fn recent_patch_ids(repo: &Repo, count: u32) -> Result<Vec<CommitPatch>> {
    let range = format!("-{count}");
    let log = repo.run(&["log", &range, "--pretty=%H %s"])?;
    if !log.success {
        return Ok(Vec::new());
    }
    recent_patch_ids_from_log(repo, &log.stdout)
}

pub fn recent_patch_ids_from_log(repo: &Repo, log_stdout: &str) -> Result<Vec<CommitPatch>> {
    let mut result = Vec::new();
    for line in log_stdout.lines() {
        let Some((hash, subject)) = line.split_once(' ') else {
            continue;
        };
        let show = repo.run(&["show", hash])?;
        let patch_id_out = repo.run_piped(&["patch-id", "--stable"], &show.stdout)?;
        let patch_id = patch_id_out
            .stdout_trimmed()
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string();
        result.push(CommitPatch {
            hash: hash.to_string(),
            patch_id,
            subject: subject.to_string(),
        });
    }
    Ok(result)
}
