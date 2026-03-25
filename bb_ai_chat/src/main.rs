use bb_ai_chat::run;
use eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    run()?;

    Ok(())
}
