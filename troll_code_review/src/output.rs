use std::process::Command;
use eyre::Context;

pub fn say_outloud(content: String) -> eyre::Result<()> {
    Command::new("say")
        .arg(content)
        .output()
        .context("Speaking out loud")?;

    Ok(())
}
