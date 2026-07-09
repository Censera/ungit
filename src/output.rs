use owo_colors::OwoColorize;

/// An in-progress action. Printed to stdout, no trailing newline semantics
/// beyond a normal line.
pub fn step(msg: impl AsRef<str>) {
    println!("{} {}", "[i]".blue().bold(), msg.as_ref());
}

/// A completed, successful outcome.
pub fn success(msg: impl AsRef<str>) {
    println!("{} {}", "[k]".green().bold(), msg.as_ref());
}

/// A non-fatal problem. Command continues.
pub fn warning(msg: impl AsRef<str>) {
    println!("{} {}", "[w]".yellow().bold(), msg.as_ref());
}

/// A fatal problem. Printed to stderr.
pub fn error(msg: impl AsRef<str>) {
    eprintln!("{} {}", "[e]".red().bold(), msg.as_ref());
}

/// A neutral informational line, no symbol, slightly dimmed.
pub fn info(msg: impl AsRef<str>) {
    println!("  {}", msg.as_ref());
}

/// Indented detail line under a step/success/warning, e.g. "next steps".
pub fn detail(msg: impl AsRef<str>) {
    println!("    {}", msg.as_ref());
}
