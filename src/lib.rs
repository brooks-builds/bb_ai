mod api;
mod context;
pub mod tools;

use std::fmt::Display;

use crate::{
    api::send_to_ai,
    context::{ChatContext, Message},
    tools::{list_files, read_file::ReadFileTool},
};
use async_openai::{Client, config::OpenAIConfig};
use eyre::{Context, Result};
use serde_json::Value;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub async fn run(
    mut user_prompt: UnboundedReceiver<String>,
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
        let Some(prompt) = user_prompt.recv().await else {
            break;
        };

        context.add_message(Message::new_user(prompt));

        let llm_response = send_to_ai(&client, &context)
            .await
            .context("Sending to ai")?;

        if let Some(content) = llm_response.content.as_ref() {
            response
                .send(AgentResponse {
                    message: Some(content.to_owned()),
                    finished: false,
                })
                .context("Sending response back to user")?;
        }

        context.add_message(llm_response.clone());

        while let Some(Some(tool_calls)) = context
            .messages
            .last()
            .map(|message| message.tool_calls.clone())
        {
            for tool_call in tool_calls {
                let tool_name = tool_call.function.name.as_str();
                let arguments = &tool_call.function.arguments;
                let id = tool_call.id.clone();
                let result = match tool_name {
                    list_files::TOOL_NAME => list_files::run_tool(arguments, id),
                    ReadFileTool::definition() => 
                    _ => Message::new_tool(
                        format!("Error, tool with name {tool_name} doesn't exist."),
                        id,
                    ),
                };

                context.add_message(result);
            }

            let llm_tool_response = send_to_ai(&client, &context)
                .await
                .context("Sending to ai after running tools")?;

            if let Some(content) = llm_tool_response.content.as_ref() {
                response
                    .send(AgentResponse {
                        message: Some(content.clone()),
                        finished: false,
                    })
                    .context("sending ai tool response content to user")?;
            }

            context.add_message(llm_tool_response);
        }

        response.send(AgentResponse {
            message: None,
            finished: true,
        })?;
    }

    Ok(())
}

pub struct AgentResponse {
    pub message: Option<String>,
    pub finished: bool,
}

impl Display for AgentResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = self.message.as_deref().unwrap_or_default();

        write!(f, "{message}")
    }
}
