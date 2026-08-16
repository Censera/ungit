use crate::error::Result;
use crate::git::{remote, status, Repo};
use crate::output;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct QualitySummary {
    pub work: Option<String>,
    pub saved: bool,
    pub published: bool,
}

pub fn summarize(repo: &Repo) -> Result<QualitySummary> {
    let work = status::current_branch(repo)?;
    let saved = !status::is_dirty(repo)?;
    let published = match remote::upstream_ref(repo)? {
        Some(upstream) => {
            let relation = status::ahead_behind(repo, &upstream)?;
            relation.ahead == 0 && relation.behind == 0
        }
        None => false,
    };

    Ok(QualitySummary { work, saved, published })
}

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
    output::info(format!("published: {}", if summary.published { "yes" } else { "no" }));
    Ok(())
}
