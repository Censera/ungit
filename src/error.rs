// The single error type used across the whole crate.
//
// Git module errors carry the raw stderr from the failed process.
// Commands and checks wrap those with policy-level context.
#[derive(Debug, thiserror::Error)]
pub enum UngitError {
    #[error("git {command} failed: {stderr}")]
    GitCommand { command: String, stderr: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("not a git repository (or any parent up to mount point)")]
    NotARepository,

    #[error("{0}")]
    Precondition(String),

    #[error("refused: {0}")]
    Refused(String),
}

pub type Result<T> = std::result::Result<T, UngitError>;
