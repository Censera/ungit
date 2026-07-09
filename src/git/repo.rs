use crate::error::{Result, UngitError};
use crate::git::command::GitExecutor;
use std::path::{Path, PathBuf};

/// A resolved Git repository: its working-tree root and the executor used
/// to talk to it. Every `commands::*` function takes one of these instead
/// of rederiving cwd/root
pub struct Repo<'a> {
    pub root: PathBuf,
    pub executor: &'a dyn GitExecutor,
}

impl<'a> Repo<'a> {
    /// Discover the repository containing `start_dir` (walks up, same as
    /// `git rev-parse --show-toplevel`). Fails with `NotARepository` if
    /// none is found.
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

    /// The repository's actual git directory, via `git rev-parse
    /// --git-dir`. Not always `<root>/.git` as a directory: in a linked
    /// worktree or submodule, `.git` is a file pointing elsewhere. Callers
    /// that need to read/write git-internal state (journal, rebase
    /// markers) should resolve this instead of joining `.git` by hand.
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
        // Normal repo: `rev-parse --git-dir` prints a relative path.
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
        // Linked worktree: `rev-parse --git-dir` prints an absolute path
        // into the main repo's `.git/worktrees/<name>`, which must not be
        // joined onto `root` (that would produce a nonexistent path).
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
