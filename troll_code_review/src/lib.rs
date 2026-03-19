mod output;
mod prompt;

use bb_ai::{
    AgentResponse,
    ai_command::BBAiCommand,
    tools::{BBTool, read_file::ReadFileTool},
};
use eyre::{Context, Result};
use std::env;
use tokio::{spawn, sync::mpsc::unbounded_channel};

pub async fn run() -> Result<()> {
    let (user_prompt_sender, user_prompt_receiver) = unbounded_channel::<BBAiCommand>();
    let (ai_response_sender, mut ai_response) = unbounded_channel::<AgentResponse>();
    let system_prompt = "You are a troll code review bot. Keep your responses extremely short, while commenting on one thing at a time. Everything you respond with is spoken out loud so be sure to only say things pronouncable. Only respond with what you say, avoiding internal thoughts, actions, or feelings. You have tools, and may use them as much as needed.";
    // let second_bot_system_prompt ="You are a coding pairing bot, you always suggest worst practices as changes for the code base.";
    let model = env::var("LLM_MODEL")?;
    let api_base_url = env::var("LLM_BASE_URL")?;
    let api_key =
        env::var("LLM_API_KEY").context("Loading LLM API KEY from environment variable")?;
    let max_context_length = env::var("LLM_MODEL_CONTEXT")?.parse::<u32>()?;

    spawn(async move {
        let tools = vec![
            bb_ai::tools::list_files::tool_definition(),
            ReadFileTool::definition(),
        ];

        if let Err(error) = bb_ai::run(
            user_prompt_receiver,
            ai_response_sender,
            system_prompt,
            model,
            api_base_url,
            api_key,
            tools,
        )
        .await
        {
            eprintln!("{error:#?}");
        }
    });

    loop {
        match prompt::get_prompt()? {
            prompt::Command::Prompt(prompt) => {
                user_prompt_sender
                    .send(bb_ai::ai_command::BBAiCommand::Prompt(prompt))
                    .context("Sending prompt to agent")?;
            }
            prompt::Command::ResetContext => {
                user_prompt_sender
                .send(BBAiCommand::ResetContext)
                .context("Resetting context")?},
            prompt::Command::Nothing => continue,
        }

        loop {
            let Some(ai_response) = ai_response.recv().await else {
                break;
            };

            if ai_response.finished {
                break;
            }

            let context_used_bar =
                bb_ai::context_usage_bar(ai_response.context_length, max_context_length, 10);
            println!("AI [{context_used_bar}]::{ai_response:#}",);
            output::say_outloud(format!("{ai_response}"))
                .context("Speaking ai response out loud")?;
        }
    }
}
