use crate::error::{Result, UngitError};
use std::process::Command;

/// Output of a git invocation, captured whether it succeeded or not.
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

/// Abstraction over "something that can run git commands".
///
/// `git/repo.rs`, `git/status.rs`, etc. take `&dyn GitExecutor` (or are
/// generic over `impl GitExecutor`) instead of calling `std::process::Command`
/// directly. This is the seam that lets `commands/` and `checks/` be tested
/// against a scripted fake instead of a real repository.
pub trait GitExecutor {
    /// Run `git <args>` in `cwd` and return its output regardless of exit
    /// code. Only I/O failures (git not found, etc.) are `Err`.
    fn run(&self, cwd: &std::path::Path, args: &[&str]) -> Result<GitOutput>;

    /// Like `run`, but writes `stdin` to the child's standard input first.
    fn run_piped(&self, cwd: &std::path::Path, args: &[&str], stdin: &str) -> Result<GitOutput>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemGit;

impl GitExecutor for SystemGit {
    fn run(&self, cwd: &std::path::Path, args: &[&str]) -> Result<GitOutput> {
        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(UngitError::Io)?;

        Ok(GitOutput {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            success: output.status.success(),
        })
    }

    fn run_piped(&self, cwd: &std::path::Path, args: &[&str], stdin: &str) -> Result<GitOutput> {
        use std::io::Write;
        use std::process::Stdio;

        let mut child = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(UngitError::Io)?;

        child
            .stdin
            .take()
            .expect("stdin was piped")
            .write_all(stdin.as_bytes())
            .map_err(UngitError::Io)?;

        let output = child.wait_with_output().map_err(UngitError::Io)?;

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
            self.responses.borrow_mut().pop_front().ok_or_else(|| {
                UngitError::Precondition("FakeGit: no more scripted responses".to_string())
            })
        }

        fn run_piped(
            &self,
            cwd: &std::path::Path,
            args: &[&str],
            _stdin: &str,
        ) -> Result<GitOutput> {
            self.run(cwd, args)
        }
    }
}
