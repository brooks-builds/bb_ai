use bb_ai::agent::{AgentHandle, AgentResponse};
use dotenvy::dotenv;
use eyre::{OptionExt, Result};
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
    let tools = vec![];
    let agent_handle = AgentHandle::spawn(api_base, api_key, model, None, tools);

    loop {
        let prompt = get_user_prompt()?;
        let mut response = agent_handle.send(prompt).await?;

        let AgentResponse { content, .. } =
            response.recv().await.ok_or_eyre("Getting agent response")?;

        println!("{content}");
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
