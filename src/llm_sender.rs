use crate::AppMessage;
use async_openai::{Client, config::OpenAIConfig};
use serde_json::Value;
use tokio::{
    spawn,
    sync::mpsc::{UnboundedSender, unbounded_channel},
};

pub async fn spawn_llm_sender(
    respond_to: UnboundedSender<AppMessage>,
    api_key: &str,
    api_base: &str,
) -> UnboundedSender<AppMessage> {
    let (tx, mut rx) = unbounded_channel();
    let config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(api_base);

    spawn(async move {
        let client = Client::with_config(config);

        while let Some(message) = rx.recv().await {
            let AppMessage::LlmSenderIO(value) = message else {
                continue;
            };

            let response: Value = client.chat().create_byot(value).await.unwrap();
            respond_to.send(AppMessage::LlmSenderIO(response)).unwrap();
        }
    });

    tx
}
