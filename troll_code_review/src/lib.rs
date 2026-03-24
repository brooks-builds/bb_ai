mod output;
mod prompt;

use bb_ai::{
    AgentResponse,
    ai_command::BBAiCommand,
    config::Config,
    tools::{BBTool, read_file::ReadFileTool},
};
use colored::Colorize;
use eyre::{Context, Result};
use notify::Watcher;
use std::{env, path::Path, sync::mpsc::channel};
use tokio::{spawn, sync::mpsc::unbounded_channel};

pub async fn run() -> Result<()> {
    let (agent_input_sender, agent_input_receiver) = unbounded_channel::<BBAiCommand>();
    let (ai_response_sender, mut ai_response) = unbounded_channel::<AgentResponse>();
    let system_prompt = "You are a troll code review bot.";
    // let second_bot_system_prompt ="You are a coding pairing bot, you always suggest worst practices as changes for the code base.";
    let model = env::var("LLM_MODEL")?;
    let api_base_url = env::var("LLM_BASE_URL")?;
    let api_key =
        env::var("LLM_API_KEY").context("Loading LLM API KEY from environment variable")?;
    let max_context_length = env::var("LLM_MODEL_CONTEXT")?.parse::<u32>()?;
    let (file_change_tx, mut file_change_rx) = channel::<notify::Result<notify::Event>>();
    let notify_config = notify::Config::default();

    notify_config.with_compare_contents(true);

    let mut file_watcher = notify::PollWatcher::new(file_change_tx, notify_config)
        .context("setting up file watching")?;

    file_watcher
        .watch(Path::new("."), notify::RecursiveMode::Recursive)
        .context("Watching all files recursively.")?;

    spawn(async move {
        let tools = vec![
            bb_ai::tools::list_files::tool_definition(),
            ReadFileTool::definition(),
            bb_ai::tools::append_to_file::AppendToFileTool::definition(),
        ];
        let config = Config {
            user_input: agent_input_receiver,
            response: ai_response_sender,
            system_prompt: system_prompt.to_owned(),
            model,
            api_base_url,
            api_key,
            tools,
        };

        if let Err(error) = bb_ai::run(config).await {
            eprintln!("{error:#?}");
        }
    });

    loop {
        match prompt::get_prompt(&mut file_change_rx)? {
            prompt::Command::Prompt(prompt) => {
                agent_input_sender
                    .send(bb_ai::ai_command::BBAiCommand::Prompt(prompt))
                    .context("Sending prompt to agent")?;
            }
            prompt::Command::ResetContext => agent_input_sender
                .send(BBAiCommand::ResetContext)
                .context("Resetting context")?,
            prompt::Command::Nothing => continue,
        }

        loop {
            let Some(ai_response) = ai_response.recv().await else {
                break;
            };

            if ai_response.finished {
                break;
            }

            let context_used_bar = bb_ai::utilities::context_usage_bar::context_usage_bar(
                ai_response.context_length,
                max_context_length,
                10,
            );
            let cost = if ai_response.cost < 0.85 {
                ai_response.cost.to_string().green()
            } else if ai_response.cost < 1.0 {
                ai_response.cost.to_string().yellow()
            } else {
                ai_response.cost.to_string().red()
            };

            println!("AI [{context_used_bar}](${cost})::{ai_response:#}",);
            output::say_outloud(format!("{ai_response}"))
                .context("Speaking ai response out loud")?;
        }
    }
}
