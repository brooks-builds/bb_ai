use crate::{
    llm_sender::LlmSenderHandle,
    tools::{Tool},
};
use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage, ChatCompletionTools,
};
use eyre::{Context, Result};
use tokio::sync::{mpsc, oneshot};

pub struct Agent {
    receiver: mpsc::Receiver<AgentMessage>,
    llm_sender_handle: LlmSenderHandle,
    messages: Vec<ChatCompletionRequestMessage>,
    model: String,
    tools: Vec<Box<dyn Tool + Send + 'static>>,
}

impl Agent {
    pub fn new(
        receiver: mpsc::Receiver<AgentMessage>,
        llm_sender_handle: LlmSenderHandle,
        model: String,
        tools: Vec<Box<dyn Tool + Send + 'static>>,
    ) -> Self {
        let messages = vec![];

        Self {
            receiver,
            llm_sender_handle,
            messages,
            model,
            tools,
        }
    }

    async fn handle_message(&mut self, message: AgentMessage) -> eyre::Result<()> {
        match message {
            AgentMessage::SendMessage { respond_to, prompt } => {
                self.messages.push(ChatCompletionRequestMessage::User(async_openai::types::chat::ChatCompletionRequestUserMessage { content: async_openai::types::chat::ChatCompletionRequestUserMessageContent::Text(prompt), ..Default::default() }));

                let tools = if self.tools.is_empty() {
                    None
                } else {
                    Some(self.tools.iter().map(|tool| tool.definition()).collect())
                };
                let response = self
                    .llm_sender_handle
                    .send_message(self.messages.clone(), self.model.clone(), tools)
                    .await?;
                let content = response.choices[0].message.content.clone().unwrap();
                let message = ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
                    content: Some(async_openai::types::chat::ChatCompletionRequestAssistantMessageContent::Text(content.clone())),
                    ..Default::default()
                });

                self.messages.push(message);
                respond_to.send(content).unwrap();

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

pub enum AgentMessage {
    SendMessage {
        respond_to: oneshot::Sender<String>,
        prompt: String,
    },
}

pub struct AgentHandle {
    sender: mpsc::Sender<AgentMessage>,
}

impl AgentHandle {
    pub fn new(
        llm_sender_handle: LlmSenderHandle,
        model: String,
        tools: Vec<Box<dyn Tool + Send + 'static>>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(1);
        let actor = Agent::new(rx, llm_sender_handle, model, tools);

        tokio::spawn(actor.run());

        Self { sender: tx }
    }

    pub async fn send_message(&self, prompt: String) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        let message = AgentMessage::SendMessage {
            respond_to: tx,
            prompt,
        };
        let _ = self.sender.send(message).await;

        rx.await.context("Sending message to agent actor")
    }
}
