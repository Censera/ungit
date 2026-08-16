use crate::error::Result;
use crate::git::{Repo, remote, status};
use crate::output;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StatusSummary {
    pub work: Option<String>,
    pub saved: bool,
    pub synced: bool,
}

/// Describes the repository in terms of the ungit workflow rather than Git internals.
pub fn summarize(repo: &Repo) -> Result<StatusSummary> {
    let work = status::current_branch(repo)?;
    let saved = status::porcelain(repo)?.is_empty();
    let synced = match remote::upstream_ref(repo)? {
        Some(upstream) => status::ahead_behind(repo, &upstream)?
            .map(|ab| ab.ahead == 0 && ab.behind == 0)
            .unwrap_or(false),
        None => false,
    };

    Ok(StatusSummary { work, saved, synced })
}

/// `ungit status`
pub fn run(repo: &Repo, json: bool) -> Result<()> {
    let summary = summarize(repo)?;

    if json {
        return output::json(&summary);
    }

    match &summary.work {
        Some(name) => output::info(format!("work: {name}")),
        None => output::info("work: none"),
    }
    output::info(format!("saved: {}", if summary.saved { "yes" } else { "no" }));
    output::info(format!("synced: {}", if summary.synced { "yes" } else { "no" }));

    Ok(())
}
