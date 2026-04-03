use crate::tools::ToolMessage;
use async_openai::{Client, config::OpenAIConfig};
use eyre::{Context, Ok, OptionExt, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    #[serde(skip)]
    tool_tx: Option<mpsc::Sender<ToolMessage>>,
    tools: Vec<Value>,
}

impl Agent {
    pub fn new(
        receiver: mpsc::Receiver<AgentMessage>,
        api_base: String,
        api_key: String,
        model: String,
        tool_tx: Option<mpsc::Sender<ToolMessage>>,
        tools: Vec<Value>,
        system_prompt: String,
    ) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(api_base)
            .with_api_key(api_key);
        let client = Client::with_config(config);
        let messages = vec![LlmMessage::new_system(system_prompt)];

        Self {
            receiver,
            client,
            model,
            messages,
            tool_tx,
            tools,
        }
    }

    pub async fn run(mut self) -> eyre::Result<()> {
        while let Some(message) = self.receiver.recv().await {
            match message {
                AgentMessage::SendMessage { prompt, respond_to } => {
                    self.handle_send_message(prompt, respond_to).await?
                }
            }
        }

        Ok(())
    }

    pub async fn handle_send_message(
        &mut self,
        prompt: String,
        respond_to: mpsc::Sender<AgentResponse>,
    ) -> eyre::Result<()> {
        let user_message = LlmMessage::new_user(prompt);

        self.messages.push(user_message);
        self.send_to_ai().await?;

        while let Some(message) = self.messages.last_mut().cloned()
            && let Some(tool_calls) = message.tool_calls.as_ref()
            && let Some(tool_tx) = self.tool_tx.as_ref()
            && !tool_calls.is_empty()
        {
            self.respond(&message, respond_to.clone(), false).await?;

            for tool_call in tool_calls.clone() {
                let name = tool_call.function.name.clone();
                let arguments = tool_call.function.arguments.clone();
                let (tool_call_tx, tool_call_rx) = oneshot::channel();
                let tool_message = ToolMessage {
                    name,
                    arguments,
                    send_to: tool_call_tx,
                };

                tool_tx.clone().send(tool_message).await?;

                let tool_call_result = tool_call_rx.await?;
                let tool_message = LlmMessage::new_tool(tool_call_result, tool_call.id.clone());

                self.messages.push(tool_message);
            }

            self.send_to_ai().await?;
        }

        let message = self.messages.last().ok_or_eyre("No messages exist")?;
        self.respond(message, respond_to, true).await?;

        Ok(())
    }

    async fn send_to_ai(&mut self) -> Result<()> {
        let mut response: LlmResponse = self.client.chat().create_byot(&self).await?;
        let choice = response
            .choices
            .first_mut()
            .ok_or_eyre("choice missing from llm response")?;

        self.messages.push(choice.message.clone());

        Ok(())
    }

    async fn respond(
        &self,
        message: &LlmMessage,
        tx: mpsc::Sender<AgentResponse>,
        finished: bool,
    ) -> Result<()> {
        let Some(content) = message.content.as_ref().cloned() else {
            return Ok(());
        };

        if content.is_empty() {
            return Ok(());
        }

        let message = AgentResponse { content, finished };

        tx.send(message).await.context("sending agent response")?;

        Ok(())
    }
}

enum AgentMessage {
    SendMessage {
        prompt: String,
        respond_to: mpsc::Sender<AgentResponse>,
    },
}

pub struct AgentHandle {
    sender: mpsc::Sender<AgentMessage>,
}

impl AgentHandle {
    pub fn spawn(
        api_base: String,
        api_key: String,
        model: String,
        tool_tx: Option<mpsc::Sender<ToolMessage>>,
        tools: Vec<Value>,
        system_prompt: String,
    ) -> Self {
        let (tx, rx) = mpsc::channel(1);
        let agent = Agent::new(rx, api_base, api_key, model, tool_tx, tools, system_prompt);
        let _handle = tokio::spawn(agent.run());

        Self { sender: tx }
    }

    pub async fn send(&self, prompt: String) -> eyre::Result<mpsc::Receiver<AgentResponse>> {
        let (tx, rx) = mpsc::channel(10);
        let message = AgentMessage::SendMessage {
            prompt,
            respond_to: tx,
        };

        self.sender.send(message).await?;

        Ok(rx)
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct LlmMessage {
    role: LlmMessageRole,
    content: Option<String>,
    tool_calls: Option<Vec<LlmResponseToolCall>>,
    tool_call_id: Option<String>,
}

impl LlmMessage {
    pub fn new_user(content: String) -> Self {
        let role = LlmMessageRole::User;
        let tool_calls = None;

        Self {
            role,
            content: Some(content),
            tool_calls,
            tool_call_id: None,
        }
    }

    pub fn new_tool(content: String, id: String) -> Self {
        let role = LlmMessageRole::Tool;

        Self {
            role,
            content: Some(content),
            tool_calls: None,
            tool_call_id: Some(id),
        }
    }

    pub fn new_system(content: String) -> Self {
        let role = LlmMessageRole::System;

        Self {
            role,
            content: Some(content),
            tool_calls: None,
            tool_call_id: None,
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
    message: LlmMessage,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LlmResponseToolCall {
    #[serde(rename = "type")]
    tool_type: String,
    id: String,
    function: LlmResponseToolCallFunction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LlmResponseToolCallFunction {
    name: String,
    arguments: String,
}

pub struct AgentResponse {
    pub content: String,
    pub finished: bool,
}
