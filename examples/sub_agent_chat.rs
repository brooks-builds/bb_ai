use bb_ai::{
    agent::{AgentHandle, AgentResponse},
    tools::{
        ToolMessage, git_diff::GitDiffHandle, git_status::GitStatusHandle,
        list_files::ListFileHandle, read_file::ReadFileHandle, spawn_agent::SpawnAgentHandle,
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
    let spawn_agent_tool = SpawnAgentHandle::spawn()?;
    let list_files_tool = ListFileHandle::spawn()?;
    let tools = vec![
        git_diff_tool.definition(),
        git_status_tool.definition(),
        read_file_tool.definition(),
        spawn_agent_tool.definition(),
        list_files_tool.definition(),
    ];
    let mut system_prompt = r#"You are a Agent controller who has a full list of tools available to you.

Your primary strategy is to delegate work to specialized sub agents. This keeps your context window clean and allows you to provide better answers. When faced with a task:

1. Break down complex requests into focused sub-tasks
2. Create specialized sub agents for each focused task (finding files, reading code, analyzing, etc.)
3. Summarize and synthesize the sub agent results for the user
4. Keep your own responses brief and high-level

You answer correctly and truthfully, keeping your responses short but accurate.

Remember: Delegation is your strength, not a limitation. More agents = better results."#.to_owned();

    system_prompt.push_str(&read_file_tool.system_prompt());
    system_prompt.push_str(&spawn_agent_tool.system_prompt());
    system_prompt.push_str(&list_files_tool.system_prompt());

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
        list_files_tool,
    ));

    loop {
        let prompt = get_user_prompt()?;
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

async fn handle_tool_calls(
    mut rx: mpsc::Receiver<ToolMessage>,
    git_diff: GitDiffHandle,
    git_status: GitStatusHandle,
    read_file: ReadFileHandle,
    spawn_agent: SpawnAgentHandle,
    tool_tx: mpsc::Sender<ToolMessage>,
    list_files: ListFileHandle,
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
                } else if tool_name == list_files.name() {
                    tools.push(list_files.definition());
                    system_prompt.push(list_files.system_prompt());
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
        } else if message.name == list_files.name() {
            let list_files = list_files.clone();

            spawn(async move {
                println!("{}", "Running list files".green());
                let result = list_files.send(&message.arguments).await.unwrap();
                message.send_to.send(result).unwrap();
            });
        }
    }

    Ok(())
}

fn get_user_prompt() -> Result<String> {
    print!("> ");
    stdout().flush()?;
    let mut prompt = String::new();
    stdin().read_line(&mut prompt)?;
    Ok(prompt)
}
