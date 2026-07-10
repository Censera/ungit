use crate::git::{remote, Repo};

pub fn suggest(repo: &Repo, command: &str, stderr: &str) -> Option<String> {
    // 1. HTTPS → SSH rewrite (auth failure)
    if let Some(fix) = suggest_https_to_ssh(repo, command, stderr) {
        let remote_name = extract_remote(command, repo);
        if remote_name == "upstream" {
            return Some(format!(
                "Warning: 'upstream' is conventionally the source repository, not a push target. Did you mean 'origin'?\n\
If you still intend to push to upstream, run: {fix}"
            ));
        }
        return Some(fix);
    }

    // 2. askpass / portal failures
    if contains_any(stderr, &["askpass", "ksshaskpass", "org.freedesktop.portal"]) {
        return Some("git config --global core.askPass \"\"".to_string());
    }

    // 3. generic SSH publickey error
    if stderr.contains("Permission denied (publickey)") {
        return Some("ssh-add -l".to_string());
    }

    // 4. explicit permission denial for the authenticated user
    if stderr.contains("Permission to") && stderr.contains("denied to") {
        return Some(
            "You don't have write access to this repository. \
Check that the remote URL matches your fork by running: git remote -v"
.to_string(),
        );
    }

    // 5. host resolution failure
    if stderr.contains("Could not resolve host") {
        return Some("check your network connection and DNS".to_string());
    }

    // 6. repository not found
    if stderr.contains("Repository not found") {
        return Some(format!(
            "git remote get-url {}",
            extract_remote(command, repo)
        ));
    }

    None
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

fn suggest_https_to_ssh(repo: &Repo, command: &str, stderr: &str) -> Option<String> {
    let is_auth_failure = stderr.contains("Authentication failed")
    || stderr.contains("No anonymous write access")
    || stderr.contains("could not read Username")
    || stderr.contains("could not read Password");
    if !is_auth_failure {
        return None;
    }
    let remote_name = extract_remote(command, repo);
    let url = remote::get_url(repo, &remote_name).ok().flatten()?;
    let ssh_url = https_github_to_ssh(&url)?;
    Some(format!("git remote set-url {remote_name} {ssh_url}"))
}

fn extract_remote(command: &str, repo: &Repo) -> String {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    if let Some(pos) = tokens.iter().position(|t| *t == "-u") {
        if let Some(name) = tokens.get(pos + 1) {
            return name.to_string();
        }
    }
    if tokens.len() >= 3 && (tokens.get(1) == Some(&"fetch") || tokens.get(1) == Some(&"pull")) {
        if let Some(name) = tokens.get(2) {
            if !name.starts_with('-') {
                return name.to_string();
            }
        }
    }
    remote::upstream_ref(repo)
    .ok()
    .flatten()
    .and_then(|full| full.split('/').next().map(str::to_string))
    .unwrap_or_else(|| "origin".to_string())
}

fn https_github_to_ssh(url: &str) -> Option<String> {
    let rest = url.strip_prefix("https://github.com/")?;
    let rest = rest.strip_suffix(".git").unwrap_or(rest);
    let mut parts = rest.splitn(2, '/');
    let owner = parts.next()?;
    let repo_name = parts.next()?;
    if owner.is_empty() || repo_name.is_empty() {
        return None;
    }
    Some(format!("git@github.com:{owner}/{repo_name}.git"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::command::test_support::FakeGit;
    use std::path::Path;

    #[test]
    fn https_to_ssh_with_dot_git_suffix() {
        assert_eq!(
            https_github_to_ssh("https://github.com/user/repo.git"),
                   Some("git@github.com:user/repo.git".to_string())
        );
    }

    #[test]
    fn https_to_ssh_without_dot_git_suffix() {
        assert_eq!(
            https_github_to_ssh("https://github.com/user/repo"),
                   Some("git@github.com:user/repo.git".to_string())
        );
    }

    #[test]
    fn https_to_ssh_rejects_non_github_host() {
        assert_eq!(
            https_github_to_ssh("https://gitlab.com/user/repo.git"),
                   None
        );
    }

    #[test]
    fn https_to_ssh_rejects_already_ssh_url() {
        assert_eq!(
            https_github_to_ssh("git@github.com:user/repo.git"),
                   None
        );
    }

    #[test]
    fn suggest_https_auth_failure_rewrites_to_ssh() {
        let git = FakeGit::new();
        git.push_ok("https://github.com/user/repo.git\n");
        let repo = Repo {
            root: Path::new("/repo").to_path_buf(),
            executor: &git,
        };
        let fix = suggest(
            &repo,
            "git push -u origin main",
            "fatal: Authentication failed for 'https://github.com/user/repo.git/'",
        );
        assert_eq!(
            fix,
            Some("git remote set-url origin git@github.com:user/repo.git".to_string())
        );
    }

    #[test]
    fn suggest_no_tty_for_username_prompt_rewrites_to_ssh() {
        let git = FakeGit::new();
        git.push_ok("https://github.com/user/repo.git\n");
        let repo = Repo {
            root: Path::new("/repo").to_path_buf(),
            executor: &git,
        };
        let fix = suggest(
            &repo,
            "git push -u origin main",
            "fatal: could not read Username for 'https://github.com': No such device or address",
        );
        assert_eq!(
            fix,
            Some("git remote set-url origin git@github.com:user/repo.git".to_string())
        );
    }

    #[test]
    fn suggest_https_auth_failure_upstream_warns_and_rewrites() {
        let git = FakeGit::new();
        git.push_ok("https://github.com/user/repo.git\n");
        let repo = Repo {
            root: Path::new("/repo").to_path_buf(),
            executor: &git,
        };
        let fix = suggest(
            &repo,
            "git push -u upstream main",
            "fatal: Authentication failed for 'https://github.com/user/repo.git/'",
        );
        assert_eq!(
            fix,
            Some(
                "Warning: 'upstream' is conventionally the source repository, not a push target. Did you mean 'origin'?\n\
If you still intend to push to upstream, run: git remote set-url upstream git@github.com:user/repo.git"
.to_string()
            )
        );
    }

    #[test]
    fn suggest_permission_denied_to_user() {
        let git = FakeGit::new();
        let repo = Repo {
            root: Path::new("/repo").to_path_buf(),
            executor: &git,
        };
        let fix = suggest(
            &repo,
            "git push upstream master",
            "ERROR: Permission to user/repo.git denied to someuser.",
        );
        assert_eq!(
            fix,
            Some(
                "You don't have write access to this repository. \
Check that the remote URL matches your fork by running: git remote -v"
.to_string()
            )
        );
    }
}
