use bb_ai::{
    agent::{BBAgent, SubAgentChannels, run_agent},
    ai_command::BBAiCommand,
    config::Config,
    context::Message,
    tools::{BBTool, git_diff::GitDiffTool, git_status::GitStatusTool, read_file::ReadFileTool},
};
use colored::Colorize;
use eyre::Result;
use serde::Deserialize;
use std::env;
use tokio::sync::mpsc::UnboundedSender;

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
    let diff_agent_tool = DiffAgentTool(diff_agent_input_tx);
    let diff_agent = BBAgent::new(diff_agent_config);

    let commit_message_agent = BBAgent::new(commit_message_agent_config);

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

struct DiffAgentTool(UnboundedSender<BBAiCommand>);

impl BBTool for DiffAgentTool {
    type Arguments = DiffAgentToolArgs;

    fn definition() -> serde_json::Value {
        serde_json::json!({
          "type": "function",
          "function": {
            "name": "diff_subagent",
            "description": "diff_subagent is an ai agent that can use tools to summarize new and changed code in the codebase. It's response is the summary.",
            "parameters": {
              "type": "object",
              "properties": {
                "prompt": {
                  "type": "string",
                  "description": "The prompt to send to the diff_subagent."
                }
              },
              "required": ["prompt"]
            }
          }
        })
    }

    fn run(&mut self, args: &str, id: String) -> std::result::Result<Message, Message> {
        let args = match serde_json::from_str::<Self::Arguments>(args) {
            Ok(args) => args,
            Err(error) => {
                eprintln!(
                    "{}",
                    format!("Error running diff agent tool: {error}").red()
                );
                return Err(Message::new_tool(format!("{error:?}"), id));
            }
        };

        todo!()
    }
}

#[derive(Debug, Deserialize)]
pub struct DiffAgentToolArgs {}
