use std::env;

use bb_ai::{
    config::Config,
    tools::{BBTool, git_diff::GitDiffTool, git_status::GitStatusTool, read_file::ReadFileTool},
};
use tokio::{spawn, sync::mpsc::unbounded_channel};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    let (user_input_tx, user_input_rx) = unbounded_channel();
    let (agent_response_tx, mut agent_response_rx) = unbounded_channel();
    let config = Config {
        user_input: user_input_rx,
        response: agent_response_tx,
        system_prompt: "You are a senior developer who loves to answer in single sentence responses. You also love puns and emoji to clarify what you are attempting to purvey.".to_owned(),
        model: env::var("LLM_MODEL")?,
        api_base_url: env::var("LLM_BASE_URL")?,
        api_key: env::var("LLM_API_KEY")?,
        tools: vec![
            GitDiffTool::definition(),
            GitStatusTool::definition(),
            ReadFileTool::definition(),
        ],
    };

    spawn(async move {
        bb_ai::run(config).await.ok();
    });

    user_input_tx.send(bb_ai::ai_command::BBAiCommand::Prompt(
        "Generate a git commit message. Read the files to ensure that you get a proper commit message. Only respond with the message after you've read the diffs and files. Keep your response to 25 words or less.".to_owned(),
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
