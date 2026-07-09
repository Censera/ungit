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

/// Resets the last commit while keeping modifications staged.
pub fn undo_last_soft(repo: &Repo) -> Result<()> {
    repo.require(&["reset", "--soft", "HEAD^"])?;
    Ok(())
}

/// Retrieves the subject line of the specified commit reference.
pub fn subject(repo: &Repo, rev: &str) -> Result<String> {
    let output = repo.require(&["log", "-1", "--pretty=%s", rev])?;
    Ok(output.stdout_trimmed().to_string())
}

/// Metadata mapping a specific commit to its computed cryptographic patch identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitPatch {
    pub hash: String,
    pub patch_id: String,
    pub subject: String,
}

/// Computes stable patch identifiers for historical commits reachable from HEAD.
pub fn recent_patch_ids(repo: &Repo, count: u32) -> Result<Vec<CommitPatch>> {
    let range = format!("-{count}");
    let log = repo.run(&["log", &range, "--pretty=%H %s"])?;
    if !log.success {
        return Ok(Vec::new());
    }
    recent_patch_ids_from_log(repo, &log.stdout)
}

/// Evaluates git diff content lines to resolve stable structural patch IDs.
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
