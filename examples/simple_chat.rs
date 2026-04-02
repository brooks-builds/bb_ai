use bb_ai::agent::AgentHandle;
use dotenvy::dotenv;
use eyre::Result;
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
