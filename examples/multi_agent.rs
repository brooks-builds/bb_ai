use std::env;

use bb_ai::{
    agent::{BBAgent, SubAgentChannels, run_agent},
    config::Config,
    tools::{BBTool, git_diff::GitDiffTool, git_status::GitStatusTool, read_file::ReadFileTool},
};
use eyre::Result;

/// This will be an agent, with two sub agents. The controller parent agent
/// will run the first agent to gather the changes of files and summarize what
/// happened. This output will be sent to the github commit message agent,
/// which will craft a commit message
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let diff_agent_config = Config::new(
        "You are a helpful ai assistant who uses tools to complete tasks",
        "anthropic/claude-haiku-4.5",
        "https://openrouter.ai/api/v1",
        env::var("LLM_API_KEY")?,
        vec![
            GitDiffTool::definition(),
            GitStatusTool::definition(),
            ReadFileTool::definition(),
        ],
        vec![],
        "This agent will check for new files, and what is changed in order to summarize what is different for this git commit.",
    );
    let commit_message_agent_config = Config::new(
        "You are a helpful ai assistant who uses tools to complete tasks",
        "anthropic/claude-haiku-4.5",
        "https://openrouter.ai/api/v1",
        env::var("LLM_API_KEY")?,
        vec![],
        vec![],
        "This agent will take in a summary of changes, and create a git commit message.",
    );
    let mut controller_config = Config::new(
        "You are a git agent, you control other agents to help the user with git.",
        "anthropic/claude-haiku-4.5",
        "https://openrouter.ai/api/v1",
        env::var("LLM_API_KEY")?,
        vec![],
        vec![],
        "This agent will control up to two agents in order to craft a good commit message",
    );

    let diff_agent_input_tx = diff_agent_config.user_input_tx.take().unwrap();
    let diff_agent_response_rx = diff_agent_config.response_rx.take().unwrap();
    let diff_agent_channels = SubAgentChannels::new(
        diff_agent_input_tx,
        diff_agent_response_rx,
        diff_agent_config.description,
    );
    let diff_agent = BBAgent::new(diff_agent_config, vec![]);

    let commit_message_agent = BBAgent::new(commit_message_agent_config, vec![]);

    let user_input_tx = controller_config.user_input_tx.take().unwrap();
    let mut agent_response_rx = controller_config.response_rx.take().unwrap();
    let controller_agent = BBAgent::new(controller_config);

    run_agent(vec![diff_agent, commit_message_agent, controller_agent]).await;

    user_input_tx.send(bb_ai::ai_command::BBAiCommand::Prompt(
        "Create a commit message for changes in this repo".to_owned(),
    ))?;

    loop {
        let Ok(response) = agent_response_rx.try_recv() else {
            continue;
        };

        if let Some(message) = response.message {
            println!("{message}");
        }

        if response.finished {
            break;
        }
    }

    Ok(())
}
