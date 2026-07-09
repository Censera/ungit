use crate::error::{Result, UngitError};
use crate::git::{commit, status, Repo};
use crate::output;
use crate::util::{fs, paths};

/// `ungit save "message" [--force]`
///
/// Stages everything and commits, refusing on obvious mistakes unless
/// `force` is set: suspected secrets, unusually large files, or files
/// that are gitignored but tracked anyway.
pub fn run(repo: &Repo, message: &str, force: bool) -> Result<()> {
    let entries = status::porcelain(repo)?;
    if entries.is_empty() {
        output::step("Nothing to save.");
        return Ok(());
    }

    if !force {
        for entry in &entries {
            if paths::looks_like_secret(&entry.path) {
                return Err(UngitError::Refused(format!(
                    "{} looks like it may contain a secret, use --force to commit anyway",
                    entry.path
                )));
            }

            let full_path = repo.root.join(&entry.path);
            if fs::is_large_file(&full_path) {
                return Err(UngitError::Refused(format!(
                    "{} is unusually large, use --force to commit anyway",
                    entry.path
                )));
            }
        }
    }

    output::step("Staging changes...");
    commit::stage_all(repo)?;

    output::step("Creating commit...");
    commit::commit(repo, message)?;

    output::success(format!("Saved: {message}"));
    Ok(())
}
