use crate::allowlist;
use crate::checks::{self, CheckResult};
use crate::error::Result;
use crate::git::status::OperationState;
use crate::git::{Repo, rebase, status};
use crate::output;

/// Prompts and applies automated fixes for active repository findings.
/// Skips findings that are allowlisted or lack an automated fix strategy.
pub fn run(repo: &Repo, auto_confirm: bool, confirm: impl Fn(&str) -> bool) -> Result<()> {
    let git_dir = repo.git_dir()?;
    let allowed_names = allowlist::read(&git_dir)?;
    let is_allowed = |name: &str| allowed_names.iter().any(|n| n == name);

    let findings = checks::run_all(repo)?;
    let mut repaired_any = false;

    for finding in findings {
        if is_allowed(finding.name) {
            continue;
        }

        let (CheckResult::Error(msg) | CheckResult::Warning(msg)) = &finding.result else {
            continue;
        };

        match finding.name {
            "merge-state" => {
                if !matches!(status::operation_state(repo)?, OperationState::Rebasing) {
                    output::warning(msg.clone());
                    continue;
                }
                let prompt = "Abort the in-progress rebase? This discards rebase progress.";
                if auto_confirm || confirm(prompt) {
                    rebase::abort(repo)?;
                    output::success("Rebase aborted; branch restored to its pre-rebase state.");
                    repaired_any = true;
                } else {
                    output::warning("Left rebase in progress.");
                }
            }
            other => {
                output::warning(format!("{other}: {msg}"));
                match &finding.fix {
                    Some(fix) => output::detail(format!("fix: {fix}")),
                    None => output::detail("no automatic fix; see `ungit check`"),
                }
            }
        }
    }

    if !repaired_any {
        output::step("Nothing needed repair.");
    }
    Ok(())
}
