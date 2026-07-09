//! Repository health checks.
//! Checks inspect state without modifying the repository or writing to stdout.

pub mod detached_head;
pub mod divergence;
pub mod duplicate_patch;
pub mod ignored_files;
pub mod merge_state;
pub mod upstream;

use crate::error::Result;
use crate::git::Repo;

/// Outcome of a single check.
#[expect(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// Nothing wrong.
    Ok,
    /// Non-blocking issue requiring user attention.
    Warning(String),
    /// Critical error. Blocks execution in check mode; target for repair mode.
    Error(String),
}

impl CheckResult {
    pub fn is_error(&self) -> bool {
        matches!(self, CheckResult::Error(_))
    }
}

/// A named check result from the suite.
///
/// `name` acts as the configuration token for suppressing findings via CLI arguments.
pub struct Finding {
    pub name: &'static str,
    pub result: CheckResult,
    /// An executable command that resolves the finding.
    /// Evaluates to `None` if the resolution is ambiguous or requires user judgment.
    pub fix: Option<String>,
}

/// Run all suite checks against the repository in a stable order.
pub fn run_all(repo: &Repo) -> Result<Vec<Finding>> {
    let (result, fix) = detached_head::check(repo)?;
    let detached_head = Finding {
        name: "detached-head",
        result,
        fix,
    };

    let (result, fix) = upstream::check(repo)?;
    let upstream = Finding {
        name: "upstream",
        result,
        fix,
    };

    let (result, fix) = divergence::check(repo)?;
    let divergence = Finding {
        name: "divergence",
        result,
        fix,
    };

    let (result, fix) = merge_state::check(repo)?;
    let merge_state = Finding {
        name: "merge-state",
        result,
        fix,
    };

    let (result, fix) = ignored_files::check(repo)?;
    let ignored_files = Finding {
        name: "ignored-files",
        result,
        fix,
    };

    let (result, fix) = duplicate_patch::check(repo)?;
    let duplicate_patch = Finding {
        name: "duplicate-patch",
        result,
        fix,
    };

    Ok(vec![
        detached_head,
        upstream,
        divergence,
        merge_state,
        ignored_files,
        duplicate_patch,
    ])
}
