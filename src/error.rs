/// The single error type used across the whole crate.
///
/// `git` module errors carry the raw stderr from the failed process.
/// `commands` and `checks` wrap those with policy-level context.
#[derive(Debug, thiserror::Error)]
pub enum UngitError {
    /// A git subprocess exited non-zero. Carries the command run and stderr.
    #[error("git {command} failed: {stderr}")]
    GitCommand { command: String, stderr: String },

    /// The git binary could not be spawned at all (not installed, not on PATH).
    #[error("failed to run git: {0}")]
    GitSpawn(#[from] std::io::Error),

    /// The current directory is not inside a Git repository.
    #[error("not a git repository (or any parent up to mount point)")]
    NotARepository,

    /// A precondition for a command was not met (e.g. dirty tree for `start`).
    #[error("{0}")]
    Precondition(String),

    /// A check or repair refused to proceed without confirmation.
    #[error("refused: {0}")]
    Refused(String),

    /// The journal file could not be read, written, or parsed.
    #[error("journal error: {0}")]
    Journal(String),

    /// Serializing a command's own output to JSON failed. Only reachable
    /// if a future `--json` payload type has a shape serde_json rejects
    /// (e.g. a non-string map key); none of the current payloads can hit
    /// this. Deliberately not `#[from]`: `journal.rs` also produces
    /// `serde_json::Error` in places where it wants the richer
    /// `Journal(String)` context instead, and one source type mapping to
    /// two variants needs to stay an explicit choice at each call site.
    #[error("failed to serialize output as JSON: {0}")]
    JsonOutput(serde_json::Error),

    /// A git child process was spawned without the stdin pipe `run_piped`
    /// requires. Not reachable through `SystemGit` (it always requests
    /// `Stdio::piped()` immediately before this is checked); exists so a
    /// future change to that spawn configuration fails loudly here
    /// instead of panicking three lines away.
    #[error("git child process has no stdin handle (piping was misconfigured)")]
    GitStdinUnavailable,

    /// One or more `checks::run_all` findings was `CheckResult::Error`.
    /// `commands::check` uses this to make the process exit non-zero
    /// after it has already printed each finding.
    #[error("one or more checks reported an error")]
    ChecksFailed,

    /// Test-only: a `FakeGit` in `git::command::test_support` received
    /// more `run`/`run_piped` calls than the test scripted responses
    /// for. Means the test under-specified the git interaction surface
    /// it needs to cover.
    #[cfg(test)]
    #[error("FakeGit: no more scripted responses")]
    FakeGitExhausted,
}

pub type Result<T> = std::result::Result<T, UngitError>;
