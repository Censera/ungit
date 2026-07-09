#[derive(Debug, thiserror::Error)]
pub enum UngitError {
    #[error("git {command} failed: {stderr}")]
    GitCommand { command: String, stderr: String },

    #[error("failed to run git: {0}")]
    GitSpawn(#[from] std::io::Error),

    #[error("not a git repository (or any parent up to mount point)")]
    NotARepository,

    #[error("{0}")]
    Precondition(String),

    #[error("refused: {0}")]
    Refused(String),

    #[error("journal error: {0}")]
    Journal(String),

    #[error("allowlist error: {0}")]
    Allowlist(String),

    #[error("failed to serialize output as JSON: {0}")]
    JsonOutput(serde_json::Error),

    #[error("git child process has no stdin handle (piping was misconfigured)")]
    GitStdinUnavailable,

    #[error("one or more checks reported an error")]
    ChecksFailed,

    #[cfg(test)]
    #[error("FakeGit: no more scripted responses")]
    FakeGitExhausted,
}

pub type Result<T> = std::result::Result<T, UngitError>;
