use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "ungit",
    version,
    about = "A safety layer over Git for everyday workflows."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Emit machine readable JSON instead of formatted text, where supported.
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Stage changes and create a commit, refusing obvious mistakes.
    Save(SaveArgs),

    /// Fetch, rebase onto upstream, and push. Creates upstream if missing.
    Sync(SyncArgs),

    /// Undo the last commit, keeping the working tree intact.
    Undo(UndoArgs),

    /// Revert the branch to its state before the last `sync`'s rebase.
    Unsync,

    /// Fetch, update main, and create a new branch from it.
    Start(StartArgs),

    /// Show a human readable repository summary.
    Status,

    /// Detect repository problems.
    Check(CheckArgs),

    /// Repair problems found by `check`.
    Repair(RepairArgs),
}

#[derive(clap::Args, Debug)]
pub struct SaveArgs {
    /// The commit message.
    pub message: String,

    /// Commit anyway despite warnings (large files, suspected secrets, etc).
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::Args, Debug)]
pub struct SyncArgs {
    /// Remote to sync against.
    #[arg(long, default_value = "origin")]
    pub remote: String,
}

#[derive(clap::Args, Debug)]
pub struct UndoArgs {
    /// Discard the undone commit's changes entirely instead of keeping
    /// them in the working tree. Destructive; requires confirmation.
    #[arg(long)]
    pub hard: bool,
}

#[derive(clap::Args, Debug)]
pub struct StartArgs {
    /// Name of the new branch.
    pub name: String,

    /// Base branch to start from. Defaults to the repository's default branch.
    #[arg(long)]
    pub from: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct CheckArgs {
    /// Silence a finding by name (e.g. `ignored-files`). Persists across
    /// runs until `--unallow` is used for the same name.
    #[arg(long)]
    pub allow: Option<String>,

    /// Stop silencing a previously-allowed finding.
    #[arg(long)]
    pub unallow: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct RepairArgs {
    /// Apply fixes without prompting for confirmation.
    #[arg(long)]
    pub yes: bool,
}
