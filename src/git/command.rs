use crate::error::{Result, UngitError};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

impl GitOutput {
    pub fn stdout_trimmed(&self) -> &str {
        self.stdout.trim()
    }
}

pub trait GitExecutor {
    fn run(&self, cwd: &std::path::Path, args: &[&str]) -> Result<GitOutput>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemGit;

impl GitExecutor for SystemGit {
    fn run(&self, cwd: &std::path::Path, args: &[&str]) -> Result<GitOutput> {
        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(UngitError::GitSpawn)?;

        Ok(GitOutput {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            success: output.status.success(),
        })
    }
}

pub fn require_success(
    executor: &dyn GitExecutor,
    cwd: &std::path::Path,
    args: &[&str],
) -> Result<GitOutput> {
    let output = executor.run(cwd, args)?;
    if !output.success {
        return Err(UngitError::GitCommand {
            command: format!("git {}", args.join(" ")),
            stderr: output.stderr.trim().to_string(),
        });
    }
    Ok(output)
}

#[cfg(test)]
pub mod test_support {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::path::PathBuf;

    #[derive(Default)]
    pub struct FakeGit {
        responses: RefCell<VecDeque<GitOutput>>,
        pub calls: RefCell<Vec<(PathBuf, Vec<String>)>>,
    }

    impl FakeGit {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn push_ok(&self, stdout: impl Into<String>) -> &Self {
            self.responses.borrow_mut().push_back(GitOutput {
                stdout: stdout.into(),
                stderr: String::new(),
                success: true,
            });
            self
        }

        pub fn push_err(&self, stderr: impl Into<String>) -> &Self {
            self.responses.borrow_mut().push_back(GitOutput {
                stdout: String::new(),
                stderr: stderr.into(),
                success: false,
            });
            self
        }
    }

    impl GitExecutor for FakeGit {
        fn run(&self, cwd: &std::path::Path, args: &[&str]) -> Result<GitOutput> {
            self.calls.borrow_mut().push((
                cwd.to_path_buf(),
                args.iter().map(|s| s.to_string()).collect(),
            ));
            self.responses
                .borrow_mut()
                .pop_front()
                .ok_or(UngitError::FakeGitExhausted)
        }
    }
}
