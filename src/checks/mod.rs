//! Repository health checks. Each check inspects one thing and returns a
//! `CheckResult`, it never mutates the repository and never prints.
//! `commands::check` runs the set and renders results, `commands::repair`
//! runs (a subset of) the same checks to decide what needs fixing.

pub mod detached_head;
pub mod divergence;
pub mod duplicate_patch;
pub mod ignored_files;
pub mod merge_state;
pub mod upstream;

use crate::error::Result;
use crate::git::Repo;

/// Outcome of a single check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// Nothing wrong.
    Ok,
    /// Worth the user's attention, but not blocking.
    Warning(String),
    /// A real problem; `check` should exit nonzero and `repair` should
    /// offer to fix it
    Error(String),
}

impl CheckResult {
    pub fn is_error(&self) -> bool {
        matches!(self, CheckResult::Error(_))
    }
}

/// A named check result, as produced by running the full suite.
pub struct Finding {
    pub name: &'static str,
    pub result: CheckResult,
}

/// Run every check against `repo` and return their findings in a fixed,
/// stable order (so `--json` output and terminal output always agree).
pub fn run_all(repo: &Repo) -> Result<Vec<Finding>> {
    Ok(vec![
        Finding {
            name: "detached-head",
            result: detached_head::check(repo)?,
        },
        Finding {
            name: "upstream",
            result: upstream::check(repo)?,
        },
        Finding {
            name: "divergence",
            result: divergence::check(repo)?,
        },
        Finding {
            name: "merge-state",
            result: merge_state::check(repo)?,
        },
        Finding {
            name: "ignored-files",
            result: ignored_files::check(repo)?,
        },
        Finding {
            name: "duplicate-patch",
            result: duplicate_patch::check(repo)?,
        },
    ])
}
