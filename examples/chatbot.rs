use bb_ai::{
    config::Config,
    utilities::{get_user_prompt::get_user_input, print_response_message::print_ai_response},
};
use std::env;
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    let (user_input_tx, user_input_rx) = unbounded_channel();
    let (agent_tx, mut agent_rx) = unbounded_channel();
    let system_prompt = "You are a helpful assistant";
    let model = "anthropic/claude-haiku-4.5";
    let api_base_url = "https://openrouter.ai/api/v1";
    let api_key = env::var("LLM_API_KEY")?;
    let tools = vec![];
    let agent_config = Config {
        user_input: user_input_rx,
        response: agent_tx,
        system_prompt: system_prompt.to_owned(),
        model: model.to_owned(),
        api_base_url: api_base_url.to_owned(),
        api_key,
        tools,
    };

    tokio::spawn(async move {
        bb_ai::run(agent_config).await.ok();
    });

    loop {
        let prompt = get_user_input()?;
        user_input_tx.send(bb_ai::ai_command::BBAiCommand::Prompt(prompt))?;

        loop {
            let Ok(agent_response) = agent_rx.try_recv() else {
                continue;
            };

            print_ai_response(&agent_response);

            if agent_response.finished {
                break;
            }
        }
    }
}
