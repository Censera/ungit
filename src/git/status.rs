use crate::error::Result;
use crate::git::repo::Repo;

/// A parsed row from a porcelain status invocation containing raw change keys and paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    pub index: char,
    pub worktree: char,
    pub path: String,
}

/// Parses the raw repository porcelain status output.
pub fn porcelain(repo: &Repo) -> Result<Vec<StatusEntry>> {
    let output = repo.require(&["status", "--porcelain=v1", "--untracked-files=all"])?;
    Ok(output
        .stdout
        .lines()
        .filter(|l| l.len() > 3)
        .map(|line| StatusEntry {
            index: line.as_bytes()[0] as char,
            worktree: line.as_bytes()[1] as char,
            path: line[3..].to_string(),
        })
        .collect())
}

/// Evaluates if there are any staged, unstaged, or untracked changes.
pub fn is_dirty(repo: &Repo) -> Result<bool> {
    Ok(!porcelain(repo)?.is_empty())
}

/// Resolves the currently active branch name. Returns `None` if the context is detached.
pub fn current_branch(repo: &Repo) -> Result<Option<String>> {
    let output = repo.run(&["rev-parse", "--abbrev-ref", "HEAD"])?;
    if !output.success {
        let symbolic = repo.run(&["symbolic-ref", "--short", "-q", "HEAD"])?;
        return Ok(if symbolic.success {
            Some(symbolic.stdout_trimmed().to_string())
        } else {
            None
        });
    }
    let name = output.stdout_trimmed().to_string();
    if name == "HEAD" {
        Ok(None)
    } else {
        Ok(Some(name))
    }
}

/// Direct tracking metrics comparing divergence counts between references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AheadBehind {
    pub ahead: u32,
    pub behind: u32,
}

/// Compares the HEAD pointer context against a target tracking reference.
pub fn ahead_behind(repo: &Repo, other_ref: &str) -> Result<Option<AheadBehind>> {
    let output = repo.run(&[
        "rev-list",
        "--left-right",
        "--count",
        &format!("HEAD...{other_ref}"),
    ])?;
    if !output.success {
        return Ok(None);
    }
    let trimmed = output.stdout_trimmed();
    let mut parts = trimmed.split_whitespace();
    let ahead = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let behind = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    Ok(Some(AheadBehind { ahead, behind }))
}

/// Internal operational transaction states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationState {
    Clean,
    Rebasing,
    Merging,
    CherryPicking,
    Reverting,
    BisectInProgress,
}

/// Inspects local metadata configurations to detect state operations like merges or rebases.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::command::test_support::FakeGit;
    use std::path::Path;

    #[test]
    fn porcelain_parses_lines() {
        let git = FakeGit::new();
        git.push_ok(" M src/main.rs\n?? new_file.rs\n");
        let repo = Repo {
            root: Path::new("/repo").to_path_buf(),
            executor: &git,
        };
        let entries = porcelain(&repo).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].worktree, 'M');
        assert_eq!(entries[1].path, "new_file.rs");
    }

    #[test]
    fn current_branch_detached() {
        let git = FakeGit::new();
        git.push_ok("HEAD\n");
        let repo = Repo {
            root: Path::new("/repo").to_path_buf(),
            executor: &git,
        };
        assert_eq!(current_branch(&repo).unwrap(), None);
    }

    #[test]
    fn ahead_behind_parses() {
        let git = FakeGit::new();
        git.push_ok("2\t3\n");
        let repo = Repo {
            root: Path::new("/repo").to_path_buf(),
            executor: &git,
        };
        let ab = ahead_behind(&repo, "@{upstream}").unwrap().unwrap();
        assert_eq!(ab.ahead, 2);
        assert_eq!(ab.behind, 3);
    }
}
