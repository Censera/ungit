use owo_colors::OwoColorize;

pub fn step(msg: impl AsRef<str>) {
    println!("  {}", msg.as_ref().blue().bold());
}

pub fn success(msg: impl AsRef<str>) {
    println!("  {}", msg.as_ref().green().bold());
}

pub fn warning(msg: impl AsRef<str>) {
    println!("  {}", msg.as_ref().yellow().bold());
}

pub fn error(msg: impl AsRef<str>) {
    eprintln!("  {}", msg.as_ref().red().bold());
}

pub fn info(msg: impl AsRef<str>) {
    println!("  {}", msg.as_ref());
}

pub fn json<T: serde::Serialize>(value: &T) -> crate::error::Result<()> {
    let rendered = serde_json::to_string_pretty(value)
        .map_err(crate::error::UngitError::JsonOutput)?;
    println!("{rendered}");
    Ok(())
}
