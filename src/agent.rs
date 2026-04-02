use serde::{Serialize};
use tokio::sync::{mpsc, oneshot};

#[derive(Serialize)]
struct Agent {
    #[serde(skip)]
    receiver: mpsc::Receiver<AgentMessage>,
}

impl Agent {
    pub fn new(receiver: mpsc::Receiver<AgentMessage>) -> Self {
        Self {
            receiver,
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

    pub async fn handle_send_message(&self, prompt: String, respond_to: oneshot::Sender<String>) -> eyre::Result<()> {
        let response = format!("AI: {prompt}");

        respond_to.send(response).expect("Agent responding");

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
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(1);
        let agent = Agent::new(rx);
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
