use bb_ai::config::Config;
use eyre::Result;
use std::{
    env,
    io::{Write, stdin, stdout},
};
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

            if let Some(message) = agent_response.message {
                let cost = agent_response.cost;
                let context = agent_response.context_length;

                println!("AI (context: {context} ${cost}){message}");
            }

            if agent_response.finished {
                break;
            }
        }
    }
}

fn get_user_input() -> Result<String> {
    print!("> ");
    stdout().flush()?;

    let mut prompt = String::new();
    stdin().read_line(&mut prompt)?;

    Ok(prompt)
}
