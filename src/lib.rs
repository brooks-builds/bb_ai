pub mod agent;
pub mod ai_command;
mod api;
pub mod config;
pub mod context;
pub mod tools;
pub mod utilities;

use crate::{
    ai_command::BBAiCommand,
    api::send_to_ai,
    config::Config,
    context::{ChatContext, Message},
};
use async_openai::{Client, config::OpenAIConfig};
use eyre::{Context, Result};
use std::fmt::Display;

pub async fn run(mut config: Config) -> Result<()> {
    let mut context = ChatContext::new(&config);
    let openai_config = OpenAIConfig::new()
        .with_api_key(&config.api_key)
        .with_api_base(&config.api_base_url);
    let client = Client::with_config(openai_config);

    loop {
        if let Some(command) = config.user_input.recv().await {
            match command {
                ai_command::BBAiCommand::Prompt(prompt) => {
                    context.add_message(Message::new_user(prompt), 0, 0.0);
                }
                BBAiCommand::ResetContext => {
                    context.reset(config.system_prompt.clone());
                    config.response.send(AgentResponse {
                        message: None,
                        finished: true,
                        context_length: context.tokens_used(),
                        cost: context.cost,
                    })?;
                    continue;
                }
            }
        }

        let llm_response = send_to_ai(&client, &context)
            .await
            .context("Sending to ai")?;

        if let Some(content) = llm_response.message.content.as_ref() {
            config
                .response
                .send(AgentResponse {
                    message: Some(content.to_owned()),
                    finished: false,
                    context_length: context.tokens_used(),
                    cost: context.cost,
                })
                .context("Sending response back to user")?;
        }

        context.add_message(
            llm_response.message.clone(),
            llm_response.tokens,
            llm_response.cost,
        );

        while let Some(Some(tool_calls)) = context
            .messages
            .last()
            .map(|message| message.tool_calls.clone())
        {
            for tool_call in tool_calls {
                let tool_name = tool_call.function.name.as_str();
                let arguments = &tool_call.function.arguments;
                let id = tool_call.id.clone();
                let result = match tools::run_tools(arguments, id, tool_name) {
                    Ok(message) => message,
                    Err(message) => message,
                };

                context.add_message(result, 0, 0.0);
            }

            let llm_tool_response = send_to_ai(&client, &context)
                .await
                .context("Sending to ai after running tools")?;

            if let Some(content) = llm_tool_response.message.content.as_ref() {
                config
                    .response
                    .send(AgentResponse {
                        message: Some(content.clone()),
                        finished: false,
                        context_length: context.tokens_used(),
                        cost: context.cost,
                    })
                    .context("sending ai tool response content to user")?;
            }

            context.add_message(
                llm_tool_response.message,
                llm_tool_response.tokens,
                llm_tool_response.cost,
            );
        }

        config.response.send(AgentResponse {
            message: None,
            finished: true,
            context_length: context.tokens_used(),
            cost: context.cost,
        })?;
    }
}

#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub message: Option<String>,
    pub finished: bool,
    pub context_length: u32,
    pub cost: f32,
}

impl Display for AgentResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = self.message.as_deref().unwrap_or_default();

        write!(f, "{message}")
    }
}
