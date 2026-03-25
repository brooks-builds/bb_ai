use bb_ai::{
    config::Config,
    utilities::{get_user_prompt::get_user_input, print_response_message::print_ai_response},
};
use std::env;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    let system_prompt = "You are a helpful assistant";
    let model = "anthropic/claude-haiku-4.5";
    let api_base_url = "https://openrouter.ai/api/v1";
    let api_key = env::var("LLM_API_KEY")?;
    let tools = vec![];
    let mut agent_config = Config::new(
        system_prompt,
        model,
        api_base_url,
        api_key,
        tools,
        vec![],
        "Simple chatbot that doesn't use any tools, allowing the user to chat with a model directly.",
    );
    let user_input_tx = agent_config.user_input_tx.take().unwrap();
    let mut agent_rx = agent_config.response_rx.take().unwrap();

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
