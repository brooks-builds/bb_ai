use bb_ai::{agent::AgentHandle, tools::random_number::RandomNumberHandle};
use dotenvy::dotenv;
use eyre::Result;
use tokio::sync::mpsc;
use std::{
    env,
    io::{Write, stdin, stdout},
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv()?;

    let api_base = env::var("LLM_BASE_URL")?;
    let api_key = env::var("LLM_API_KEY")?;
    let model = env::var("LLM_MODEL")?;
    let (tool_tx, mut tool_rx) = mpsc::channel(10);
    let random_number_tool = RandomNumberHandle::spawn();
    let agent_handle = AgentHandle::spawn(api_base, api_key, model);

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
