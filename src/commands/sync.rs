use crate::error::{Result, UngitError};
use crate::git::{Repo, rebase, remote, status};
use crate::journal;
use crate::output;

/// Fetches remote tracking data, rebases the active branch, and pushes updates.
/// Publishes the branch to the remote target if no upstream tracking relation is defined.
pub fn run(repo: &Repo, remote_name: &str) -> Result<()> {
    let Some(branch) = status::current_branch(repo)? else {
        return Err(UngitError::Precondition(
            "HEAD is detached; switch to a branch before syncing".to_string(),
        ));
    };

    output::step(format!("Fetching {remote_name}..."));
    remote::fetch(repo, Some(remote_name))?;

    match remote::upstream_ref(repo)? {
        None => {
            output::step(format!("No upstream for '{branch}'; publishing it now."));
            remote::push(repo, remote_name, &branch, true)?;
            output::success(format!("Branch '{branch}' published to {remote_name}."));
            return Ok(());
        }
        Some(upstream) => {
            // Track the pre-rewrite HEAD reference.
            let pre_rebase_sha = repo
                .require(&["rev-parse", "HEAD"])?
                .stdout_trimmed()
                .to_string();

            output::step(format!("Rebasing onto {upstream}..."));
            let result = rebase::onto(repo, &upstream)?;

            if !result.success {
                let conflicts = rebase::conflicted_paths(repo)?;
                output::error("Rebase stopped due to conflicts.");
                for path in &conflicts {
                    output::detail(format!("conflict: {path}"));
                }
                output::info("Resolve the conflicts above, then run:");
                output::detail("ungit save <msg> (then: ungit sync after)");
                output::detail("git add <file>   (or: for each resolved file)");
                output::detail("ungit sync       (or: git rebase --continue)");
                output::info("To abandon the rebase instead:");
                output::detail("git rebase --abort");
                return Err(UngitError::Refused(
                    "rebase stopped (resolve conflicts before syncing again)".to_string(),
                ));
            }

            let git_dir = repo.git_dir()?;
            journal::record(
                &git_dir,
                journal::Entry {
                    operation: journal::Operation::Sync,
                    branch: branch.clone(),
                    pre_image_sha: pre_rebase_sha,
                    recorded_at_unix: 0,
                },
            )?;
        }
    }

    output::step(format!("Pushing to {remote_name}..."));
    remote::push(repo, remote_name, &branch, false)?;

    output::success("Repository is up to date.");
    Ok(())
}
