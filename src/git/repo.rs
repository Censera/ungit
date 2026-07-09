use crate::error::{Result, UngitError};
use crate::git::command::GitExecutor;
use std::path::{Path, PathBuf};

/// A resolved Git repository context tracking working-tree root paths and execution bindings.
pub struct Repo<'a> {
    pub root: PathBuf,
    pub executor: &'a dyn GitExecutor,
}

impl<'a> Repo<'a> {
    /// Attempts to locate the top-level repository context by traversing directories upward.
    pub fn discover(executor: &'a dyn GitExecutor, start_dir: &Path) -> Result<Self> {
        let output = executor.run(start_dir, &["rev-parse", "--show-toplevel"])?;
        if !output.success {
            return Err(UngitError::NotARepository);
        }
        let root = PathBuf::from(output.stdout_trimmed());
        Ok(Repo { root, executor })
    }

    pub fn run(&self, args: &[&str]) -> Result<crate::git::command::GitOutput> {
        self.executor.run(&self.root, args)
    }

    pub fn require(&self, args: &[&str]) -> Result<crate::git::command::GitOutput> {
        crate::git::command::require_success(self.executor, &self.root, args)
    }

    pub fn run_piped(&self, args: &[&str], stdin: &str) -> Result<crate::git::command::GitOutput> {
        self.executor.run_piped(&self.root, args, stdin)
    }

    /// Resolves the canonical internal git storage or structural routing path directory.
    pub fn git_dir(&self) -> Result<PathBuf> {
        let output = self.require(&["rev-parse", "--git-dir"])?;
        let raw = PathBuf::from(output.stdout_trimmed());
        Ok(if raw.is_absolute() {
            raw
        } else {
            self.root.join(raw)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::command::test_support::FakeGit;

    #[test]
    fn discover_ok() {
        let git = FakeGit::new();
        git.push_ok("/home/user/project\n");
        let repo = Repo::discover(&git, Path::new("/home/user/project/src")).unwrap();
        assert_eq!(repo.root, PathBuf::from("/home/user/project"));
    }

    #[test]
    fn discover_not_a_repo() {
        let git = FakeGit::new();
        git.push_err("fatal: not a git repository");
        let result = Repo::discover(&git, Path::new("/tmp"));
        assert!(result.is_err());
        match result {
            Err(UngitError::NotARepository) => {}
            Err(other) => panic!("expected NotARepository, got a different error: {other}"),
            Ok(_) => panic!("expected an error, got Ok"),
        }
    }

    #[test]
    fn git_dir_joins_relative_output() {
        let git = FakeGit::new();
        git.push_ok(".git\n");
        let repo = Repo {
            root: PathBuf::from("/home/user/project"),
            executor: &git,
        };
        assert_eq!(
            repo.git_dir().unwrap(),
            PathBuf::from("/home/user/project/.git")
        );
    }

    #[test]
    fn git_dir_keeps_absolute_output() {
        let git = FakeGit::new();
        git.push_ok("/home/user/project/.git/worktrees/feature\n");
        let repo = Repo {
            root: PathBuf::from("/home/user/project-feature-worktree"),
            executor: &git,
        };
        assert_eq!(
            repo.git_dir().unwrap(),
            PathBuf::from("/home/user/project/.git/worktrees/feature")
        );
    }
}
