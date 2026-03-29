use async_openai::{Client, config::OpenAIConfig, types::chat::{ChatCompletionRequestMessage, CreateChatCompletionResponse}};
use eyre::{Context, Result};
use tokio::sync::{mpsc::{Receiver, Sender, channel}, oneshot};

pub struct LlmSender {
    receiver: Receiver<LlmSenderMessage>,
    client: Client<OpenAIConfig>,
}

impl LlmSender {
    pub fn new(config: OpenAIConfig, receiver: Receiver<LlmSenderMessage>) -> Self {
        let client = Client::with_config(config);

        Self {
            receiver,
            client,
        }
    }

    async fn handle_message(&mut self, message: LlmSenderMessage) -> eyre::Result<()> {
        match message {
            LlmSenderMessage::SendMessage {
                model,
                messages,
                respond_to,
            } => {
                let request = async_openai::types::chat::CreateChatCompletionRequest {
                    messages,
                    model,
                    ..Default::default()
                };
                let response = self.client.chat().create(request).await?;

                respond_to.send(response).unwrap();
                
                Ok(())
            }
        }
    }

    async fn run(mut self) -> Result<()> {
        while let Some(message) = self.receiver.recv().await {
            self.handle_message(message).await?;
        }

        Ok(())
    }

}


pub enum LlmSenderMessage {
    SendMessage {
        respond_to: oneshot::Sender<CreateChatCompletionResponse>,
        model: String,
        messages: Vec<ChatCompletionRequestMessage>,
    }
}

pub struct LlmSenderHandle {
    sender: Sender<LlmSenderMessage>,
}

impl LlmSenderHandle {
    pub fn new(api_base: &str, api_key: &str) -> Self {
        let (tx, rx) = channel(8);
        let config = OpenAIConfig::new().with_api_base(api_base).with_api_key(api_key);
        let actor = LlmSender::new(config, rx);

        tokio::spawn(actor.run());

        Self {
            sender: tx,
        }
    }

    pub async fn send_message(&self, messages: Vec<ChatCompletionRequestMessage>, model: String) -> Result<CreateChatCompletionResponse> {
        let (tx, rx) = oneshot::channel();
        let message = LlmSenderMessage::SendMessage {
            respond_to: tx,
            model,
            messages,
        };
        let _ = self.sender.send(message).await;
        
        rx.await.context("sending message to LlmRequest Actor")

    }
}
