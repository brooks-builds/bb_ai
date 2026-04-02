use async_openai::{Client, config::OpenAIConfig};
use eyre::{Ok, OptionExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

pub const NAME: &str = "agent";

#[derive(Serialize)]
struct Agent {
    #[serde(skip)]
    receiver: mpsc::Receiver<AgentMessage>,
    #[serde(skip)]
    client: Client<OpenAIConfig>,
    model: String,
    messages: Vec<LlmMessage>,
}

impl Agent {
    pub fn new(receiver: mpsc::Receiver<AgentMessage>, api_base: String, api_key: String, model: String) -> Self {
        let config = OpenAIConfig::new().with_api_base(api_base).with_api_key(api_key);
        let client = Client::with_config(config);
        let messages = vec![];
        
        Self {
            receiver,
            client,
            model,
            messages,
        }
    }

    pub async fn run(mut self) -> eyre::Result<()> {
        while let Some(message) = self.receiver.recv().await {
            match message {
                AgentMessage::SendMessage { prompt, respond_to } => self.handle_send_message(prompt, respond_to).await?,
            }
        }

        Ok(())
    }

    pub async fn handle_send_message(&mut self, prompt: String, respond_to: oneshot::Sender<String>) -> eyre::Result<()> {
        let user_message = LlmMessage::new_user(prompt);

        self.messages.push(user_message);

        let mut response: LlmResponse = self.client.chat().create_byot(&self).await?;
        let choice = response.choices.first_mut().ok_or_eyre("choice missing from llm response")?;

        self.messages.push(choice.message.clone());

        if let Some(content) = choice.message.content.take() {
            respond_to.send(content).unwrap();
        }

        Ok(())
    }
}

enum AgentMessage {
    SendMessage {
        prompt: String,
        respond_to: oneshot::Sender<String>,
    }
}

pub struct AgentHandle {
    sender: mpsc::Sender<AgentMessage>,
}

impl AgentHandle {
    pub fn spawn(api_base: String, api_key: String, model: String) -> Self {
        let (tx, rx) = mpsc::channel(1);
        let agent = Agent::new(rx, api_base, api_key, model);
        let _handle = tokio::spawn(agent.run());

        Self {
            sender: tx
        }
    }

    pub async fn send(&self, prompt: String) -> eyre::Result<String> {
        let (tx, rx) = oneshot::channel();
        let message = AgentMessage::SendMessage { prompt, respond_to: tx };

        self.sender.send(message).await?;

        let response = rx.await?;

        Ok(response)
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct LlmMessage {
    role: LlmMessageRole,
    content: Option<String>,
}

impl LlmMessage {
    pub fn new_user(content: String) -> Self {
        let role = LlmMessageRole::User;

        Self {
            role,
            content: Some(content),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
enum LlmMessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Deserialize)]
struct LlmResponse {
    pub choices: Vec<LlmMessageResponseChoice>,
}

#[derive(Deserialize)]
struct LlmMessageResponseChoice {
    message: LlmMessage
}
