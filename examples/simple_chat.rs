use std::{env, io::{Write, stdin, stdout}};
use bb_ai::{agent::AgentHandle, llm_sender::LlmSenderHandle};
use dotenvy::dotenv;
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    color_eyre::install()?;
    
    let api_base = env::var("LLM_BASE_URL")?;
    let api_key = env::var("LLM_API_KEY")?;
    let model = env::var("LLM_MODEL")?;
    let llm_sender_handle = LlmSenderHandle::new(&api_base, &api_key);
    let agent_handle = AgentHandle::new(llm_sender_handle, model);

    loop {
        let prompt = get_prompt()?;
        let response = agent_handle.send_message(prompt).await?;
        println!("{response}");
    }
}

fn get_prompt() -> Result<String> {
    print!("> ");
    stdout().flush()?;

    let mut prompt = String::new();

    stdin().read_line(&mut prompt)?;

    Ok(prompt.trim().to_owned())
}
