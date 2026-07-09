use crate::checks::CheckResult;
use crate::error::Result;
use crate::git::Repo;

/// Warns if any file that is normally gitignored is nonetheless tracked
/// (a common source of "why is this huge binary in my repo" surprises),
/// or if untracked-but-ignored files exist that look like they were
/// accidentally staged before. This check only reports; `save` is the one
/// that refuses to commit over an ignoredfile surprise.
pub fn check(repo: &Repo) -> Result<CheckResult> {
    // `git ls-files -i -c --exclude-standard` lists tracked files that
    // also match an ignore rulen i.e. someone ran `git add -f` on
    // something .gitignore says shouldn't be tracked.
    let output = repo.require(&["ls-files", "-i", "-c", "--exclude-standard"])?;
    let offenders: Vec<&str> = output.stdout.lines().collect();

    if offenders.is_empty() {
        Ok(CheckResult::Ok)
    } else if offenders.len() == 1 {
        Ok(CheckResult::Warning(format!(
            "tracked file matches a gitignore rule: {}",
            offenders[0]
        )))
    } else {
        Ok(CheckResult::Warning(format!(
            "{} tracked files match gitignore rules (e.g. {})",
            offenders.len(),
            offenders[0]
        )))
    }
}
