use bb_ai::{
    agent::{AgentHandle, AgentResponse},
    tools::{
        ToolMessage, git_diff::GitDiffHandle, git_status::GitStatusHandle,
        read_file::ReadFileHandle,
    },
};
use colored::Colorize;
use dotenvy::dotenv;
use eyre::{Context, OptionExt, Result};
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
    let read_file_tool = ReadFileHandle::spawn()?;
    let tools = vec![
        git_diff_tool.definition(),
        git_status_tool.definition(),
        read_file_tool.definition(),
    ];
    let mut system_prompt = r#"You are a precise tool-calling assistant with access to git tools. Follow these rules strictly:

1. When given a task, consider ALL available tools and determine which combination gives the most complete answer.
2. For questions about repository changes, use BOTH git status (to see which files are affected and their staging state) AND git diff (to see the actual content changes). Neither alone gives the full picture.
3. After each tool response, evaluate whether you have enough information to fully answer the user's question. If not, call additional tools before responding.
4. When the task's goal IS fully met, stop calling tools and respond with a clear summary.
5. Never call a tool after you have all the information needed.
6. Always report the final result clearly."#.to_owned();

    system_prompt.push_str(&read_file_tool.system_prompt());

    let agent_handle = AgentHandle::spawn(
        api_base,
        api_key,
        model,
        Some(tool_tx),
        tools,
        system_prompt,
    );

    spawn(handle_tool_calls(
        tool_rx,
        git_diff_tool,
        git_status_tool,
        read_file_tool,
    ));

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
    read_file: ReadFileHandle,
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

            message
                .send_to
                .send(result)
                .expect("Sending tool call result to agent");
        } else if message.name == read_file.name() {
            println!("{}", "read file tool running".green());
            let result = read_file
                .send(message.arguments)
                .await
                .context("Running read file tool")?;
            message
                .send_to
                .send(result)
                .expect("Error sending read file tool back to agent");
        }
    }

    Ok(())
}
