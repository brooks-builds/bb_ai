use crate::{llm_sender::LlmSenderHandle, tools::Tool};
use async_openai::types::chat::{
    ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage, ChatCompletionResponseMessage
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
                let Some(choice) = response.choices.first() else {
                    return Ok(());
                };
                let ChatCompletionResponseMessage {
                    content,
                    refusal: _,
                    tool_calls,
                    annotations,
                    role,
                    audio,
                    ..
                } = &choice.message;

                if let Some(tool_calls) = tool_calls.as_ref() {
                    self.handle_tool_calls(tool_calls).await?;
                }
                let message = ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
                    content: content.as_ref().map(|content| async_openai::types::chat::ChatCompletionRequestAssistantMessageContent::Text(content.to_owned())),
                    ..Default::default()
                });
                let agent_response = AgentResponse {
                    content: content.as_ref().cloned(),
                };

                self.messages.push(message);
                respond_to.send(agent_response).unwrap();

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

    async fn handle_tool_calls(&self, tool_calls: &[ChatCompletionMessageToolCalls]) -> Result<()> {
        for tool_call in tool_calls {
            match tool_call {
                async_openai::types::chat::ChatCompletionMessageToolCalls::Function(chat_completion_message_tool_call) => {
                    let id = &chat_completion_message_tool_call.id;
                    let name = &chat_completion_message_tool_call.function.name;
                    let Some(tool_handle) = self.tools.iter().find(|tool| tool.name() == name) else {
                        continue;
                    }

                    let tool_result = tool_handle.
                },
                async_openai::types::chat::ChatCompletionMessageToolCalls::Custom(chat_completion_message_custom_tool_call) => unimplemented!(),
            }

        }

        Ok(())
    }
}

pub enum AgentMessage {
    SendMessage {
        respond_to: oneshot::Sender<AgentResponse>,
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

    pub async fn send_message(&self, prompt: String) -> Result<AgentResponse> {
        let (tx, rx) = oneshot::channel();
        let message = AgentMessage::SendMessage {
            respond_to: tx,
            prompt,
        };
        let _ = self.sender.send(message).await;

        rx.await.context("Sending message to agent actor")
    }
}

#[derive(Debug)]
pub struct AgentResponse {
    pub content: Option<String>,
}
