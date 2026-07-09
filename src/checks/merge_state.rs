use crate::checks::CheckResult;
use crate::error::Result;
use crate::git::Repo;

/// Warns if any tracked files match gitignore rules.
pub fn check(repo: &Repo) -> Result<(CheckResult, Option<String>)> {
    // List tracked files that match an ignore rule.
    let output = repo.require(&["ls-files", "-i", "-c", "--exclude-standard"])?;
    let offenders: Vec<&str> = output.stdout.lines().collect();

    if offenders.is_empty() {
        Ok((CheckResult::Ok, None))
    } else if offenders.len() == 1 {
        Ok((
            CheckResult::Warning(format!(
                "tracked file matches a gitignore rule: {}",
                offenders[0]
            )),
            // A fix command is provided only when a single file is affected.
            Some(format!("git rm --cached {}", offenders[0])),
        ))
    } else {
        Ok((
            CheckResult::Warning(format!(
                "{} tracked files match gitignore rules (e.g. {})",
                offenders.len(),
                offenders[0]
            )),
            None,
        ))
    }
}
