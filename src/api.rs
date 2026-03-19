use crate::context::{ChatContext, Message};
use async_openai::{Client, config::OpenAIConfig};
use eyre::{Context, Result, bail};
use serde::Deserialize;

pub async fn send_to_ai(
    client: &Client<OpenAIConfig>,
    context: &ChatContext,
) -> Result<AiResponseMessage> {
    let response: AiResponse = client
        .chat()
        .create_byot(context)
        .await
        .context("Sending message to openai api")?;
    let Some(message) = response.choices.first().cloned() else {
        bail!("AI responded without a message");
    };

    Ok(AiResponseMessage {
        message: message.message,
        tokens: response.usage.total_tokens,
        cost: response.usage.cost,
    })
}

#[derive(Debug, Deserialize, Clone)]
pub struct AiResponse {
    pub choices: Vec<AiResponseChoice>,
    pub usage: AiUsage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AiResponseChoice {
    pub message: Message,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AiUsage {
    pub total_tokens: u32,
    pub cost: f32,
}

pub struct AiResponseMessage {
    pub message: Message,
    pub tokens: u32,
    pub cost: f32,
}
