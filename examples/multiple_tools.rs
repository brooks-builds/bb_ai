use bb_ai::{
    agent::{AgentHandle, AgentResponse},
    tools::{ToolMessage, git_diff::GitDiffHandle, git_status::GitStatusHandle},
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
    dotenv()?;

    let api_base = env::var("LLM_BASE_URL")?;
    let api_key = env::var("LLM_API_KEY")?;
    let model = env::var("LLM_MODEL")?;
    let (tool_tx, tool_rx) = mpsc::channel(10);
    let git_diff_tool = GitDiffHandle::spawn()?;
    let git_status_tool = GitStatusHandle::spawn()?;
    let tools = vec![git_diff_tool.definition(), git_status_tool.definition()];
    let agent_handle = AgentHandle::spawn(api_base, api_key, model, Some(tool_tx), tools);

    spawn(handle_tool_calls(tool_rx, git_diff_tool, git_status_tool));

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
    git_diff: GitDiffHandle,
    git_status: GitStatusHandle,
) -> Result<()> {
    while let Some(message) = rx.recv().await {
        if message.name.as_str() == "git_diff" {
            println!("{}", "git diff tool running".green());
            let result = git_diff.send().await?;

            message
                .send_to
                .send(result)
                .expect("Sending tool call result to agent");
        } else if message.name.as_str() == "git_status" {
            println!("{}", "git status tool running".green());
            let result = git_status.send().await?;
            dbg!(&result);

            message
                .send_to
                .send(result)
                .expect("Sending tool call result to agent");
        }
    }

    Ok(())
}
