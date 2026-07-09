use crate::error::Result;
use crate::git::repo::Repo;

/// A single line of `git status --porcelain=v1` output, split into its
/// two status characters and the path. Left untyped (`char`, not an enum)
/// deliberately: `git`'s own status codes are the vocabulary here, and
/// interpreting them is a `checks`/`commands` concern, not this module's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    pub index: char,
    pub worktree: char,
    pub path: String,
}

/// Raw porcelain status, parsed into entries. Includes untracked files.
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

pub fn is_dirty(repo: &Repo) -> Result<bool> {
    Ok(!porcelain(repo)?.is_empty())
}

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

/// Ahead/behind counts relative to a ref, typically `<branch>@{upstream}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AheadBehind {
    pub ahead: u32,
    pub behind: u32,
}

/// Compare HEAD against `other_ref` (e.g. `"@{upstream}"` or
/// `"origin/main"`). Returns `None` if the ref does not resolve, which is
/// the normal case for "no upstream configured".
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

/// What mid-operation state, if any, the repository is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationState {
    Clean,
    Rebasing,
    Merging,
    CherryPicking,
    Reverting,
    BisectInProgress,
}

/// Detect an in-progress rebase/merge/etc. by presence of the marker files
/// git itself uses. No git subprocess needed for this one.
pub fn operation_state(repo: &Repo) -> OperationState {
    let git_dir = repo.root.join(".git");
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
    }
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
