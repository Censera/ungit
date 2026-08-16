use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ungit", version, about = "A simple, safe workflow for Git.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Emit machine readable JSON instead of formatted text, where supported.
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Save all current changes as one commit.
    Save(SaveArgs),

    /// Update the current work and publish it.
    Update,

    /// Undo the last save while keeping its changes in the working tree.
    Undo(UndoArgs),

    /// Begin a new piece of work from the repository's default branch.
    Begin(BeginArgs),

    /// Show the health of the current workflow.
    Quality,
}

#[derive(clap::Args, Debug)]
pub struct SaveArgs {
    /// The save message.
    pub message: String,

    /// Bypass save safety checks.
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::Args, Debug)]
pub struct UndoArgs {
    /// Discard the undone save's changes permanently instead of keeping them.
    #[arg(long)]
    pub hard: bool,
}

#[derive(clap::Args, Debug)]
pub struct BeginArgs {
    /// Name of the new piece of work.
    pub name: String,
}
