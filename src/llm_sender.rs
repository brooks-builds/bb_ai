use async_openai::{Client, config::OpenAIConfig, types::chat::ChatCompletionResponseStream};
use futures::StreamExt;
use serde_json::Value;
use tokio::{
    spawn,
    sync::mpsc::{UnboundedSender, unbounded_channel},
};

use crate::AppMessage;

pub async fn spawn_llm_sender(
    respond_to: UnboundedSender<AppMessage>,
    api_key: &str,
    api_base: &str,
    stream: bool,
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

             if stream {
                let mut stream: ChatCompletionResponseStream = client.chat().create_stream_byot(value).await.unwrap();

                while let Some(result) = stream.next().await {
                    if let Ok(result) = result {
                        let value = serde_json::to_value(&result).unwrap();
                        respond_to.send(AppMessage::LlmSenderIO(value)).unwrap();
                    }
                }
            } else {
                let response: Value =client.chat().create_byot(value).await.unwrap();
                respond_to.send(AppMessage::LlmSenderIO(response)).unwrap();
            }

        }
    });

    tx
}
