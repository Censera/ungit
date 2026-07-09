mod checks;
mod cli;
mod commands;
mod error;
mod git;
mod journal;
mod output;
mod util;

use clap::Parser;
use cli::{Cli, Commands};
use git::{Repo, SystemGit};
use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let executor = SystemGit;

    let cwd = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            output::error(format!("could not determine current directory: {e}"));
            return ExitCode::FAILURE;
        }
    };

    let repo = match Repo::discover(&executor, &cwd) {
        Ok(repo) => repo,
        Err(e) => {
            output::error(e.to_string());
            return ExitCode::FAILURE;
        }
    };

    let result = match &cli.command {
        Commands::Save(args) => commands::save::run(&repo, &args.message, args.force),
        Commands::Sync(args) => commands::sync::run(&repo, &args.remote),
        Commands::Undo(args) => run_undo(&repo, args),
        Commands::Unsync => commands::unsync::run(&repo, prompt_confirm),
        Commands::Start(args) => commands::start::run(&repo, &args.name, args.from.as_deref()),
        Commands::Status => commands::status::run(&repo, cli.json),
        Commands::Check => commands::check::run(&repo, cli.json),
        Commands::Repair(args) => commands::repair::run(&repo, args.yes, prompt_confirm),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            output::error(e.to_string());
            ExitCode::FAILURE
        }
    }
}

/// `undo --hard` is destructive, so it always confirms interactively
/// regardless of any other flag, there is no `--yes` escape hatch for it
/// by design; the user types the command again if they meant it.
fn run_undo(repo: &Repo, args: &cli::UndoArgs) -> error::Result<()> {
    if args.hard && !prompt_confirm("Discard the last commit's changes permanently?") {
        return Err(error::UngitError::Refused(
            "undo --hard cancelled".to_string(),
        ));
    }
    commands::undo::run(repo, args.hard)
}

fn prompt_confirm(message: &str) -> bool {
    print!("{message} [y/N] ");
    // A failed flush only risks the prompt text appearing late relative
    // to the terminal waiting for input; it does not affect the
    // correctness of the confirmation itself, and there is nothing
    // actionable to do about it here. Not treated as fatal.
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
