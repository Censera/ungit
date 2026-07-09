const SUSPICIOUS_NAMES: &[&str] = &[
    ".env",
    ".env.local",
    "id_rsa",
    "id_ed25519",
    "credentials.json",
    "secrets.yml",
    "secrets.yaml",
];

const SUSPICIOUS_SUFFIXES: &[&str] = &[".pem", ".pfx", ".p12", ".key"];

pub fn looks_like_secret(path: &str) -> bool {
    let file_name = path.rsplit('/').next().unwrap_or(path);

    if SUSPICIOUS_NAMES.contains(&file_name) {
        return true;
    }
    SUSPICIOUS_SUFFIXES
        .iter()
        .any(|suffix| file_name.ends_with(suffix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_dotenv() {
        assert!(looks_like_secret(".env"));
        assert!(looks_like_secret("config/.env"));
    }

    #[test]
    fn detects_key_files() {
        assert!(looks_like_secret("keys/server.pem"));
        assert!(looks_like_secret("id_rsa"));
    }

    #[test]
    fn ignores_normal_files() {
        assert!(!looks_like_secret("src/main.rs"));
        assert!(!looks_like_secret("README.md"));
    }
}
