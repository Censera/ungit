mod cli;
mod commands;
mod error;
mod git;
mod output;
mod util;

use clap::Parser;
use cli::{Cli, Commands};
use git::{Repo, SystemGit};
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
        Commands::Sync => commands::sync::run(&repo),
        Commands::Undo(args) => commands::undo::run(&repo, args.hard),
        Commands::Start(args) => commands::start::run(&repo, &args.name),
        Commands::Status => commands::status::run(&repo, cli.json),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            output::error(e.to_string());
            ExitCode::FAILURE
        }
    }
}
