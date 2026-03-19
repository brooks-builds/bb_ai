pub mod ai_command;
mod api;
mod context;
pub mod tools;

use crate::{
    ai_command::BBAiCommand,
    api::send_to_ai,
    context::{ChatContext, Message},
};
use async_openai::{Client, config::OpenAIConfig};
use colored::Colorize;
use eyre::{Context, Result};
use serde_json::Value;
use std::fmt::Display;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub async fn run(
    mut user_input: UnboundedReceiver<BBAiCommand>,
    response: UnboundedSender<AgentResponse>,
    system_prompt: impl Into<String>,
    model: impl Into<String>,
    api_base_url: impl Into<String>,
    api_key: impl Into<String>,
    tools: Vec<Value>,
) -> Result<()> {
    let mut context = ChatContext::new(model, system_prompt, tools);
    let openai_config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(api_base_url);
    let client = Client::with_config(openai_config);

    loop {
        if let Some(command) = user_input.recv().await {
            match command {
                ai_command::BBAiCommand::Prompt(prompt) => {
                    context.add_message(Message::new_user(prompt), 0, 0.0);
                }
                BBAiCommand::ResetContext => {
                    context.reset();
                    response.send(AgentResponse {
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
            response
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
                response
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

        response.send(AgentResponse {
            message: None,
            finished: true,
            context_length: context.tokens_used(),
            cost: context.cost,
        })?;
    }
}

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

pub fn context_usage_bar(
    tokens_used: u32,
    max_tokens: u32,
    bar_length: u32,
) -> colored::ColoredString {
    let mut bar = String::new();
    let progress_char = '=';
    let token_used_percentage = tokens_used / max_tokens;
    let mut bar_chars = token_used_percentage * bar_length;
    let color_change = bar_length / 3;
    let bars_used = bar_chars;

    while bar_chars > 0 {
        bar.push(progress_char);
        bar_chars -= 1
    }

    bar.push('>');

    for _ in bar.len()..bar_length as usize {
        bar.push(' ');
    }

    if bars_used < color_change {
        bar.green()
    } else if bars_used > bars_used - color_change {
        bar.red()
    } else {
        bar.yellow()
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn contect_usage_bar_shows_percentage_used() {
        let tokens_used = 500;
        let max_tokens = 200000;
        let bar_length = 10;
        let expected = ">         ".to_owned();

        assert_eq!(
            context_usage_bar(tokens_used, max_tokens, bar_length),
            expected.green()
        );
    }

    #[test]
    fn contect_usage_bar_shows_green_on_low_context() {
        let tokens_used = 500;
        let max_tokens = 200000;
        let bar_length = 10;
        let expected = ">         ".to_owned();

        assert_eq!(
            context_usage_bar(tokens_used, max_tokens, bar_length),
            expected.green()
        );
    }
}
