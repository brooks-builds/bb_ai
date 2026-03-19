use eyre::{Result};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    troll_code_review::run().await?;

    Ok(())
}


