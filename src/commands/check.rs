use crate::checks::{self, CheckResult};
use crate::error::Result;
use crate::git::Repo;
use crate::output;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct JsonFinding {
    name: &'static str,
    status: &'static str,
    message: Option<String>,
}

/// `ungit check`
///
/// Runs every check in `checks::run_all` and renders the results.
/// Returns an error (nonzero exit, via `main`) if any check reported
/// `CheckResult::Error`.
pub fn run(repo: &Repo, json: bool) -> Result<()> {
    let findings = checks::run_all(repo)?;
    let has_error = findings.iter().any(|f| f.result.is_error());

    if json {
        let rendered: Vec<JsonFinding> = findings
            .iter()
            .map(|f| match &f.result {
                CheckResult::Ok => JsonFinding {
                    name: f.name,
                    status: "ok",
                    message: None,
                },
                CheckResult::Warning(msg) => JsonFinding {
                    name: f.name,
                    status: "warning",
                    message: Some(msg.clone()),
                },
                CheckResult::Error(msg) => JsonFinding {
                    name: f.name,
                    status: "error",
                    message: Some(msg.clone()),
                },
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&rendered).unwrap());
    } else {
        for finding in &findings {
            match &finding.result {
                CheckResult::Ok => output::success(format!("{}: OK", finding.name)),
                CheckResult::Warning(msg) => output::warning(format!("{}: {}", finding.name, msg)),
                CheckResult::Error(msg) => output::error(format!("{}: {}", finding.name, msg)),
            }
        }
    }

    if has_error {
        Err(crate::error::UngitError::Precondition(
            "the specific check failure message".to_string(),
        ))
    } else {
        Ok(())
    }
}
