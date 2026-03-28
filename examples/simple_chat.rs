use bb_ai::{AppMessage, agent::spawn_agent};
use dotenvy::dotenv;
use eyre::Result;
use std::{
    env,
    io::{Write, stdin, stdout},
};
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    dotenv()?;

    let (tx, mut rx) = unbounded_channel::<AppMessage>();
    let system_prompt = "You are a friendly, helpful chatbot.";
    let model = "anthropic/claude-haiku-4.5";
    let api_key = env::var("LLM_API_KEY")?;
    let api_base = env::var("LLM_BASE_URL")?;
    let stream = false;
    let agent_tx = spawn_agent(
        tx,
        system_prompt.to_owned(),
        model.to_owned(),
        api_key,
        api_base,
        stream,
    )
    .await;
    let context_window = 200_000.0;

    loop {
        let prompt = get_user_prompt()?;

        agent_tx.send(AppMessage::AgentIO {
            content: prompt,
            cost: None,
            tokens_used: None,
        })?;

        let Some(response) = rx.recv().await else {
            break;
        };

        match response {
            AppMessage::AgentIO {
                content,
                cost,
                tokens_used,
            } => println!(
                "${} (context: {}%) {content}",
                cost.unwrap(), (tokens_used.unwrap() as f32 / context_window) * 100.0
            ),
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn get_user_prompt() -> Result<String> {
    print!("> ");
    stdout().flush()?;

    let mut prompt = String::new();

    stdin().read_line(&mut prompt)?;

    Ok(prompt.trim().to_owned())
}
