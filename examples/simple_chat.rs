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
    let agent_tx = spawn_agent(
        tx,
        system_prompt.to_owned(),
        model.to_owned(),
        api_key,
        api_base,
    )
    .await;

    loop {
        let prompt = get_user_prompt()?;

        agent_tx.send(AppMessage::AgentIn(prompt))?;

        loop {
            let Some(response) = rx.recv().await else {
                break;
            };

            match response {
                AppMessage::AgentOut { content, finished } => {
                    if finished {
                        println!("{content}");
                        break;
                    } else {
                        print!("{content}");
                        stdout().flush()?;
                        continue;
                    }
                }
                _ => unreachable!(),
            }
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
