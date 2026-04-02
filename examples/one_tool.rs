use bb_ai::{
    agent::AgentHandle,
    tools::{ToolMessage, random_number::RandomNumberHandle},
};
use colored::Colorize;
use dotenvy::dotenv;
use eyre::Result;
use std::{
    env,
    io::{Write, stdin, stdout},
};
use tokio::{spawn, sync::mpsc};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv()?;

    let api_base = env::var("LLM_BASE_URL")?;
    let api_key = env::var("LLM_API_KEY")?;
    let model = env::var("LLM_MODEL")?;
    let (tool_tx, tool_rx) = mpsc::channel(10);
    let random_number_tool = RandomNumberHandle::spawn();
    let tools = vec![random_number_tool.definition()];
    let agent_handle = AgentHandle::spawn(api_base, api_key, model, Some(tool_tx), tools);

    spawn(handle_tool_calls(tool_rx, random_number_tool));

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

async fn handle_tool_calls(
    mut rx: mpsc::Receiver<ToolMessage>,
    random_number: RandomNumberHandle,
) -> Result<()> {
    while let Some(message) = rx.recv().await {
        if message.name.as_str() == "random_number" {
            println!("{}", "random number tool running".green());
            let result = random_number.send(&message.arguments).await?;
            println!("{}", format!("random number is {result}").green());

            message
                .send_to
                .send(result)
                .expect("Sending tool call result to agent");
        }
    }

    Ok(())
}
