mod api;
mod context;
pub mod tools;

use crate::{
    api::send_to_ai,
    context::{ChatContext, Message},
    tools::list_files,
};
use async_openai::{Client, config::OpenAIConfig};
use eyre::{Context, Result};
use serde_json::Value;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub async fn run(
    mut user_prompt: UnboundedReceiver<String>,
    response: UnboundedSender<String>,
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

        if !llm_response.content.is_empty() {
            response
                .send(llm_response.content.clone())
                .context("Sending response back to user")?;
        }

        if let Some(tool_calls) = &llm_response.tool_calls {
            for tool_call in tool_calls {
                let tool_name = tool_call.function.name.as_str();
                let arguments = &tool_call.function.arguments;
                let id = &tool_call.id;
                let result = match tool_name {
                    list_files::TOOL_NAME => list_files::run_tool(arguments, id),
                    _ => Message::new_tool(
                        format!("Error, tool with name {tool_name} doesn't exist."),
                        id,
                    ),
                };

                dbg!(&result);

                context.add_message(result);
            }

            let llm_tool_response = send_to_ai(&client, &context)
                .await
                .context("Sending to ai after running tools")?;

            dbg!(&llm_tool_response);

            if !llm_tool_response.content.is_empty() {
                response
                    .send(llm_tool_response.content.clone())
                    .context("sending ai tool response content to user")?;
            }
        }

        context.add_message(llm_response);
    }

    Ok(())
}
