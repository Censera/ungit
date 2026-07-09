use crate::checks::{self, CheckResult};
use crate::error::Result;
use crate::git::status::OperationState;
use crate::git::{rebase, status, Repo};
use crate::output;

/// `ungit repair [--yes]`
///
/// Re-runs `checks::run_all`, and for every finding with a known
/// automatic fix, prompts before applying it (unless `yes` skips the
/// prompt). Findings without an automatic fix are reported but left
/// alone.
///
/// Note on the journal: `rebase::abort` is itself a full, correct
/// reversal of the interrupted rebase (git restores the branch to its
/// pre-rebase tip using its own bookkeeping). There is nothing for
/// `ungit`'s journal to add here, so this command does not write one
/// see `commands::sync` for the operation that actually needs it.
pub fn run(repo: &Repo, auto_confirm: bool, confirm: impl Fn(&str) -> bool) -> Result<()> {
    let findings = checks::run_all(repo)?;
    let mut repaired_any = false;

    for finding in findings {
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
                output::warning(format!("{other}: {msg} (no automatic fix; see `ungit check`)"));
            }
        }
    }

    if !repaired_any {
        output::step("Nothing needed repair.");
    }
    Ok(())
}
