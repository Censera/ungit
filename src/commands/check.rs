use crate::allowlist;
use crate::checks::{self, CheckResult};
use crate::cli::CheckArgs;
use crate::error::Result;
use crate::git::Repo;
use crate::output;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct JsonFinding {
    name: &'static str,
    status: &'static str,
    message: Option<String>,
    fix: Option<String>,
    allowed: bool,
}

/// Executes repository checks and renders findings.
/// Returns an error if any unallowed check evaluates to `CheckResult::Error`.
/// Updates the allowlist if specified by `args` and returns early.
pub fn run(repo: &Repo, json: bool, args: &CheckArgs) -> Result<()> {
    let git_dir = repo.git_dir()?;

    if let Some(name) = &args.allow {
        allowlist::add(&git_dir, name)?;
        output::success(format!(
            "'{name}' will no longer be reported by `ungit check`."
        ));
        output::detail(format!("ungit check --unallow {name}    (to re-enable it)"));
        return Ok(());
    }
    if let Some(name) = &args.unallow {
        allowlist::remove(&git_dir, name)?;
        output::success(format!("'{name}' will be reported again."));
        return Ok(());
    }

    let findings = checks::run_all(repo)?;
    let allowed_names = allowlist::read(&git_dir)?;
    let is_allowed = |name: &str| allowed_names.iter().any(|n| n == name);

    let has_unallowed_error = findings
        .iter()
        .any(|f| f.result.is_error() && !is_allowed(f.name));

    if json {
        let rendered: Vec<JsonFinding> = findings
            .iter()
            .map(|f| {
                let allowed = is_allowed(f.name);
                match &f.result {
                    CheckResult::Ok => JsonFinding {
                        name: f.name,
                        status: "ok",
                        message: None,
                        fix: None,
                        allowed,
                    },
                    CheckResult::Warning(msg) => JsonFinding {
                        name: f.name,
                        status: "warning",
                        message: Some(msg.clone()),
                        fix: f.fix.clone(),
                        allowed,
                    },
                    CheckResult::Error(msg) => JsonFinding {
                        name: f.name,
                        status: "error",
                        message: Some(msg.clone()),
                        fix: f.fix.clone(),
                        allowed,
                    },
                }
            })
            .collect();
        output::json(&rendered)?;
    } else {
        for finding in &findings {
            if is_allowed(finding.name) && !finding.result.is_error() {
                // Skip non-blocking, allowed findings.
                continue;
            }

            match &finding.result {
                CheckResult::Ok => output::success(format!("{}: ok", finding.name)),
                CheckResult::Warning(msg) => {
                    output::warning(format!("{}: {}", finding.name, msg));
                    if let Some(fix) = &finding.fix {
                        output::detail(format!("fix: {fix}"));
                    }
                    output::detail(format!("ungit check --allow {}", finding.name));
                }
                CheckResult::Error(msg) => {
                    if is_allowed(finding.name) {
                        output::warning(format!(
                            "{}: {} (allowed, not failing the command)",
                            finding.name, msg
                        ));
                    } else {
                        output::error(format!("{}: {}", finding.name, msg));
                    }
                    if let Some(fix) = &finding.fix {
                        output::detail(format!("fix: {fix}"));
                    }
                    if !is_allowed(finding.name) {
                        output::detail(format!("ungit check --allow {}", finding.name));
                    }
                }
            }
        }
    }

    if has_unallowed_error {
        Err(crate::error::UngitError::ChecksFailed)
    } else {
        Ok(())
    }
}
