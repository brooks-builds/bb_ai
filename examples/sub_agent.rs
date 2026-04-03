use bb_ai::{
    agent::{AgentHandle, AgentResponse},
    tools::{
        ToolMessage, git_diff::GitDiffHandle, git_status::GitStatusHandle,
        read_file::ReadFileHandle, spawn_agent::SpawnAgentHandle,
    },
};
use colored::Colorize;
use dotenvy::dotenv;
use eyre::{Context, OptionExt, Result};
use std::env;
use tokio::{spawn, sync::mpsc};

/// # Sub Agent Example
///
/// This example will use three agents total.
///
/// - Controller agent that will call other agents which do the actual work
/// - Sub agent to read all the changes in the current directory and summarize them
/// - Sub agent to take in changes and generate a one-line clickbait git commit message
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
    let spawn_agent_tool = SpawnAgentHandle::spawn()?;
    let tools = vec![
        git_diff_tool.definition(),
        git_status_tool.definition(),
        read_file_tool.definition(),
        spawn_agent_tool.definition(),
    ];
    let mut system_prompt = r#"You are a precise tool-calling assistant with access to git tools. Follow these rules strictly:

1. When given a task, consider ALL available tools and determine which combination gives the most complete answer.
2. For questions about repository changes, use BOTH git status (to see which files are affected and their staging state) AND git diff (to see the actual content changes). Neither alone gives the full picture.
3. After each tool response, evaluate whether you have enough information to fully answer the user's question. If not, call additional tools before responding.
4. When the task's goal IS fully met, stop calling tools and respond with a clear summary.
5. Never call a tool after you have all the information needed.
6. Always report the final result clearly."#.to_owned();

    system_prompt.push_str(&read_file_tool.system_prompt());
    system_prompt.push_str(&spawn_agent_tool.system_prompt());

    let agent_handle = AgentHandle::spawn(
        api_base,
        api_key,
        model,
        Some(tool_tx.clone()),
        tools,
        system_prompt,
    );

    spawn(handle_tool_calls(
        tool_rx,
        git_diff_tool,
        git_status_tool,
        read_file_tool,
        spawn_agent_tool,
        tool_tx.clone(),
    ));

    let prompt = "Please create a git commit message using sub agents to help out so your context remains as clean as possible. Make sure the sub agents have access to the tools they need to see what the changes are, but also see what untracked files exist (and their contents) as they are also part of the commit message. Only respond with a single sentence commit message. Please make the message as clickbaity as possible. You are allowed to break this out into as many sub agents as needed. After responding with the sub agent, write short report on how the work got completed so the user knows what sub agents were created, what tools they had access to, and who did what. As a controller ai, you don't do the work yourself, but delegate all work to sub agents, no matter how small.".to_owned();
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
    Ok(())
}

async fn handle_tool_calls(
    mut rx: mpsc::Receiver<ToolMessage>,
    git_diff: GitDiffHandle,
    git_status: GitStatusHandle,
    read_file: ReadFileHandle,
    spawn_agent: SpawnAgentHandle,
    tool_tx: mpsc::Sender<ToolMessage>,
) -> Result<()> {
    while let Some(message) = rx.recv().await {
        if message.name.as_str() == "git_diff" {
            println!("{}", "git diff tool running".green());
            let git_diff = git_diff.clone();
            spawn(async move {
                let result = git_diff.send().await.unwrap();
                message
                    .send_to
                    .send(result)
                    .expect("Sending tool call result to agent");
            });
        } else if message.name.as_str() == "git_status" {
            println!("{}", "git status tool running".green());
            let git_status = git_status.clone();

            spawn(async move {
                let result = git_status.send().await.unwrap();

                message
                    .send_to
                    .send(result)
                    .expect("Sending tool call result to agent");
            });
        } else if message.name == read_file.name() {
            println!("{}", "read file tool running".green());
            let read_file = read_file.clone();

            spawn(async move {
                let result = read_file
                    .send(message.arguments)
                    .await
                    .context("Running read file tool")
                    .unwrap();

                message
                    .send_to
                    .send(result)
                    .expect("Error sending read file tool back to agent");
            });
        } else if message.name == spawn_agent.name() {
            println!("{}", "Spawing sub agent".green());

            let spawn_agent = spawn_agent.clone();
            let tool_tx = tool_tx.clone();
            let mut tools = vec![];
            let mut system_prompt = vec![
                    "You are a helpful ai assistant who has access to several tools. Use the tools provided to you and return just the results that was asked of you.".to_owned(),
                ];
            let args = spawn_agent
                .parse_args(&message.arguments)
                .context("Parsing args when running tool")
                .unwrap();

            for tool_name in args.tools {
                if tool_name == "git_diff" {
                    tools.push(git_diff.definition());
                    system_prompt.push("You have access to a tool git_diff. This allows to you run the command `git diff` to see file changes in a git repo. You will only be able to see changes for files already tracked. If you want to see changes for untracked files, you will need to run a different tool.".to_owned());
                } else if tool_name == "git_status" {
                    tools.push(git_status.definition());
                    system_prompt.push("You have access to a tool called `git_status`. This will run the command `git status` in the repo which will tell you what files have been changes, and if there are any files that are untracked.".to_owned());
                } else if tool_name == read_file.name() {
                    tools.push(read_file.definition());
                    system_prompt.push(read_file.system_prompt());
                } else if tool_name == spawn_agent.name() {
                    tools.push(spawn_agent.definition());
                    system_prompt.push(spawn_agent.system_prompt());
                }
            }

            spawn(async move {
                let api_base = env::var("LLM_BASE_URL").unwrap();
                let api_key = env::var("LLM_API_KEY").unwrap();
                let model = env::var("LLM_MODEL").unwrap();

                let result = spawn_agent
                    .send(
                        api_base,
                        api_key,
                        model,
                        Some(tool_tx),
                        tools,
                        system_prompt.join("\n"),
                        args.prompt,
                    )
                    .await
                    .unwrap();

                message
                    .send_to
                    .send(result)
                    .expect("Error sending sub agent respons to agent");
            });
        }
    }

    Ok(())
}
