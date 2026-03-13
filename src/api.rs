use crate::context::{ChatContext, Message};
use async_openai::{Client, config::OpenAIConfig};
use eyre::{Context, Result, bail};
use serde::Deserialize;

pub async fn send_to_ai(client: &Client<OpenAIConfig>, context: &ChatContext) -> Result<Message> {
    let response: AiResponse = client
        .chat()
        .create_byot(context)
        .await
        .context("Sending message to openai api")?;
    let Some(message) = response.choices.first().cloned() else {
        bail!("AI responded without a message");
    };

    Ok(message.message)
}

#[derive(Debug, Deserialize, Clone)]
pub struct AiResponse {
    pub choices: Vec<AiResponseChoice>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AiResponseChoice {
    pub message: Message,
}
