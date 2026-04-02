use std::io::{Write, stdin, stdout};

use bb_ai::agent::AgentHandle;
use dotenvy::dotenv;
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv()?;

    let agent_handle = AgentHandle::spawn();

    loop {
        let prompt = get_user_prompt()?;

        let response = agent_handle.send(prompt).await?;

        println!("{response}");
    }
}

fn get_user_prompt() -> Result<String> {
    let mut prompt = String::new();

    print!("> ");
    stdout().flush()?;
    stdin().read_line(&mut prompt)?;

    prompt = prompt.trim().to_owned();

    Ok(prompt)
}
