use crate::error::{Result, UngitError};
use crate::git::command::GitExecutor;
use std::path::{Path, PathBuf};

/// A resolved Git repository: its working-tree root and the executor used
/// to talk to it. Every `commands::*` function takes one of these instead
/// of re-deriving cwd/root logic itself.
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
}
