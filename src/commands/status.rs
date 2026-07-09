use crate::error::Result;
use crate::git::status::OperationState;
use crate::git::{Repo, remote, status};
use crate::output;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StatusSummary {
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub dirty_files: usize,
    pub operation: &'static str,
}

fn operation_label(state: OperationState) -> &'static str {
    match state {
        OperationState::Clean => "clean",
        OperationState::Rebasing => "rebase",
        OperationState::Merging => "merge",
        OperationState::CherryPicking => "cherry-pick",
        OperationState::Reverting => "revert",
        OperationState::BisectInProgress => "bisect",
    }
}

/// Gathers everything `ungit status` reports. Split from `run` so
/// `--json` and human rendering share one source of truth.
pub fn summarize(repo: &Repo) -> Result<StatusSummary> {
    let branch = status::current_branch(repo)?;
    let upstream = remote::upstream_ref(repo)?;
    let ab = if upstream.is_some() {
        status::ahead_behind(repo, "@{upstream}")?.unwrap_or_default()
    } else {
        Default::default()
    };
    let dirty_files = status::porcelain(repo)?.len();
    let operation = operation_label(status::operation_state(repo)?);

    Ok(StatusSummary {
        branch,
        upstream,
        ahead: ab.ahead,
        behind: ab.behind,
        dirty_files,
        operation,
    })
}

/// `ungit status`
pub fn run(repo: &Repo, json: bool) -> Result<()> {
    let summary = summarize(repo)?;

    if json {
        return output::json(&summary);
    }

    match &summary.branch {
        Some(b) => output::info(format!("branch: {b}")),
        None => output::info("branch: (detached HEAD)"),
    }
    match &summary.upstream {
        Some(u) => output::info(format!("upstream: {u}")),
        None => output::info("upstream: (none)"),
    }
    output::info(format!(
        "ahead/behind: {} / {}",
        summary.ahead, summary.behind
    ));
    output::info(format!("dirty files: {}", summary.dirty_files));
    output::info(format!("operation: {}", summary.operation));

    Ok(())
}
