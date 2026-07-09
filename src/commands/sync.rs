use crate::error::{Result, UngitError};
use crate::git::{Repo, rebase, remote, status};
use crate::output;

/// `ungit sync [--remote origin]`
///
/// Fetch, rebase onto upstream, push. If there's no upstream yet, publish
/// the current branch instead of rebasing against nothing. On conflicts,
/// stop and tell the user exactly what to do; never auto-resolve.
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
            output::step(format!("Rebasing onto {upstream}..."));
            let result = rebase::onto(repo, &upstream)?;

            if !result.success {
                let conflicts = rebase::conflicted_paths(repo)?;
                output::error("Rebase stopped due to conflicts.");
                for path in &conflicts {
                    output::detail(format!("conflict: {path}"));
                }
                output::info("Resolve the conflicts above, then run:");
                output::detail("git add <file>   (for each resolved file)");
                output::detail("ungit sync       (or: git rebase --continue)");
                output::info("To abandon the rebase instead:");
                output::detail("git rebase --abort");
                return Err(UngitError::Refused(
                    "rebase stopped; resolve conflicts before syncing again".to_string(),
                ));
            }
        }
    }

    output::step(format!("Pushing to {remote_name}..."));
    remote::push(repo, remote_name, &branch, false)?;

    output::success("Repository is up to date.");
    Ok(())
}
