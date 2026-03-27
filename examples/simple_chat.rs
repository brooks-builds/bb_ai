use bb_ai::agent::Agent;
use dotenvy::dotenv;
use eyre::Result;
use std::{env, io::{Write, stdin, stdout}};
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    dotenv()?;

    let (tx, mut rx) = unbounded_channel::<String>();
    let system_prompt = "You are a friendly, helpful chatbot.";
    let model = "anthropic/claude-haiku-4.5";
    let api_key = env::var("LLM_API_KEY")?;
    let api_base = env::var("LLM_BASE_URL")?;
    let agent_handle = Agent::spawn(tx);

    loop {
        let prompt = get_user_prompt()?;

        agent_handle.send(prompt)?;

        loop {
            let Ok(response) = rx.try_recv() else {
                break;
            };

            println!("AI: {response}");
        }
    }
}

fn get_user_prompt() -> Result<String> {
    print!("> ");
    stdout().flush()?;

    let mut prompt = String::new();

    stdin().read_line(&mut prompt)?;

    Ok(prompt.trim().to_owned())
}
