//! Standard output formatting primitives for reporting application status metrics.
//!
//! Centralizes message delivery styling, coloring rules, terminal stream routing,
//! and structured JSON serialization output formatters.

use owo_colors::OwoColorize;

/// Logs an intermediate progression milestone message.
pub fn step(msg: impl AsRef<str>) {
    println!("  {} {}", "[i]".blue().bold(), msg.as_ref());
}

/// Logs a terminal successful completion event message.
pub fn success(msg: impl AsRef<str>) {
    println!("  {} {}", "[•]".green().bold(), msg.as_ref());
}

/// Logs a non-fatal anomaly message to inform the client process.
pub fn warning(msg: impl AsRef<str>) {
    println!("  {} {}", "[!]".yellow().bold(), msg.as_ref());
}

/// Logs a fatal termination event context message to standard error.
pub fn error(msg: impl AsRef<str>) {
    eprintln!("  {} {}", "[x]".red().bold(), msg.as_ref());
}

/// Logs contextual secondary information without an accompanying symbol prefix.
pub fn info(msg: impl AsRef<str>) {
    println!("      {}", msg.as_ref());
}

/// Logs minor structural trace diagnostic items or remediation suggestions.
pub fn detail(msg: impl AsRef<str>) {
    println!("        {}", msg.as_ref().dimmed());
}

/// Serializes and prints structural system states directly to standard output.
pub fn json<T: serde::Serialize>(value: &T) -> crate::error::Result<()> {
    let rendered =
        serde_json::to_string_pretty(value).map_err(crate::error::UngitError::JsonOutput)?;
    println!("{rendered}");
    Ok(())
}
