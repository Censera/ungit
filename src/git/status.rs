use crate::error::{Result, UngitError};
use crate::git::repo::Repo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    pub index: char,
    pub worktree: char,
    pub path: String,
}

pub fn porcelain(repo: &Repo) -> Result<Vec<StatusEntry>> {
    let output = repo.require(&["status", "--porcelain=v1", "--untracked-files=all"])?;
    Ok(output
        .stdout
        .lines()
        .filter(|line| line.len() > 3)
        .map(|line| StatusEntry {
            index: line.as_bytes()[0] as char,
            worktree: line.as_bytes()[1] as char,
            path: line[3..].to_string(),
        })
        .collect())
}

pub fn is_dirty(repo: &Repo) -> Result<bool> {
    Ok(!porcelain(repo)?.is_empty())
}

pub fn current_branch(repo: &Repo) -> Result<Option<String>> {
    let output = repo.require(&["rev-parse", "--abbrev-ref", "HEAD"])?;
    let name = output.stdout_trimmed();
    Ok((name != "HEAD").then(|| name.to_string()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AheadBehind {
    pub ahead: u32,
    pub behind: u32,
}

pub fn ahead_behind(repo: &Repo, other_ref: &str) -> Result<AheadBehind> {
    let output = repo.require(&[
        "rev-list",
        "--left-right",
        "--count",
        &format!("HEAD...{other_ref}"),
    ])?;
    let mut parts = output.stdout_trimmed().split_whitespace();
    let ahead = parts
        .next()
        .ok_or_else(|| UngitError::Precondition("Git returned no ahead count".to_string()))?
        .parse()
        .map_err(|error| {
            UngitError::Precondition(format!("invalid ahead count from Git: {error}"))
        })?;
    let behind = parts
        .next()
        .ok_or_else(|| UngitError::Precondition("Git returned no behind count".to_string()))?
        .parse()
        .map_err(|error| {
            UngitError::Precondition(format!("invalid behind count from Git: {error}"))
        })?;
    Ok(AheadBehind { ahead, behind })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationState {
    Clean,
    Rebasing,
    Merging,
    CherryPicking,
    Reverting,
    BisectInProgress,
}

pub fn operation_state(repo: &Repo) -> Result<OperationState> {
    let git_dir = repo.git_dir()?;
    Ok(
        if git_dir.join("rebase-merge").is_dir() || git_dir.join("rebase-apply").is_dir() {
            OperationState::Rebasing
        } else if git_dir.join("MERGE_HEAD").is_file() {
            OperationState::Merging
        } else if git_dir.join("CHERRY_PICK_HEAD").is_file() {
            OperationState::CherryPicking
        } else if git_dir.join("REVERT_HEAD").is_file() {
            OperationState::Reverting
        } else if git_dir.join("BISECT_LOG").is_file() {
            OperationState::BisectInProgress
        } else {
            OperationState::Clean
        },
    )
}
