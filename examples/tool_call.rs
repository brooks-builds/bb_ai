use std::env;

use bb_ai::{
    config::Config,
    tools::BBTool,
    utilities::{get_user_prompt::get_user_input, print_response_message::print_ai_response},
};
use tokio::{spawn, sync::mpsc::unbounded_channel};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    let (user_input_tx, user_input_rx) = unbounded_channel();
    let (agent_output_tx, mut agent_output_rx) = unbounded_channel();
    let config = Config {
        user_input: user_input_rx,
        response: agent_output_tx,
        system_prompt: "You are a helpful AI Agent".to_owned(),
        model: env::var("LLM_MODEL")?,
        api_base_url: env::var("LLM_BASE_URL")?,
        api_key: env::var("LLM_API_KEY")?,
        tools: vec![bb_ai::tools::random_number::RandomNumberTool::definition()],
        user_input_tx: None,
        response_rx: None,
        agents: vec![],
        description: "A agent that can generate real random numbers via a tool call".to_owned(),
    };

    spawn(async move {
        bb_ai::run(config).await.unwrap();
    });

    loop {
        let prompt = get_user_input()?;
        user_input_tx.send(bb_ai::ai_command::BBAiCommand::Prompt(prompt))?;

        loop {
            let Ok(response) = agent_output_rx.try_recv() else {
                continue;
            };

            print_ai_response(&response);

            if response.finished {
                break;
            }
        }
    }
}
