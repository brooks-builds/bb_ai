use bb_ai::{
    agent::{AgentHandle, AgentResponse},
    tools::{ToolMessage, random_number::RandomNumberHandle},
};
use colored::Colorize;
use dotenvy::dotenv;
use eyre::{OptionExt, Result};
use std::{
    env,
    io::{Write, stdin, stdout},
};
use tokio::{spawn, sync::mpsc};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv().ok();

    let api_base = env::var("LLM_BASE_URL")?;
    let api_key = env::var("LLM_API_KEY")?;
    let model = env::var("LLM_MODEL")?;
    let (tool_tx, tool_rx) = mpsc::channel(10);
    let random_number_tool = RandomNumberHandle::spawn();
    let tools = vec![random_number_tool.definition()];
    let system_prompt = "You are a precise tool-calling assistant. Follow these rules strictly:\n1. When given a task that requires tool calls, use the provided tools to accomplish it.\n2. After each tool response, evaluate whether the task'\''s goal has been met.\n3. If the goal IS met, immediately stop calling tools and respond to the user with a summary of what happened.\n4. If the goal is NOT yet met, make exactly one more tool call and re-evaluate.\n5. Never call a tool after the goal has been satisfied.\n6. Always report the final result clearly.".to_owned();
    let agent_handle = AgentHandle::spawn(api_base, api_key, model, Some(tool_tx), tools, system_prompt);

    spawn(handle_tool_calls(tool_rx, random_number_tool));

    loop {
        let prompt = get_user_prompt()?;

        if prompt.is_empty() {
            continue;
        }

        let mut response_rx = agent_handle.send(prompt).await?;

        loop {
            let AgentResponse { content, finished } = response_rx
                .recv()
                .await
                .ok_or_eyre("no agent response from agent handle send channel")?;

            println!("{content}");

            if finished {
                break;
            }
        }
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
