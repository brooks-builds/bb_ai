use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::Config;

#[derive(Debug, Serialize)]
pub struct ChatContext {
    pub model: String,
    pub messages: Vec<Message>,
    pub tools: Vec<Value>,
    #[serde(skip)]
    pub tokens_used: u32,
    #[serde(skip)]
    pub cost: f32,
}

impl ChatContext {
    pub fn new(config: &Config) -> Self {
        let model = config.model.clone();
        let mut messages = vec![Message::new_system(config.system_prompt.clone())];
        let tools = config.tools.clone();

        if let Some(norms) = config.norms.as_ref().cloned() {
            messages.push(Message::new_user(norms));
        }

        Self {
            model,
            messages,
            tools,
            tokens_used: 0,
            cost: 0.0,
        }
    }

    pub fn add_message(&mut self, message: Message, tokens: u32, cost: f32) {
        self.messages.push(message);
        self.tokens_used += tokens;
        self.cost += cost;
    }

    pub fn tokens_used(&self) -> u32 {
        self.tokens_used
    }

    pub fn reset(&mut self, config: &Config) {
        println!("{}", "Resetting context".blue());

        let system_prompt = Message::new_system(config.system_prompt.clone());

        self.messages.truncate(0);
        self.tokens_used = 0;
        self.messages.push(system_prompt);

        if let Some(norms) = config.norms.as_ref().cloned() {
            let message = Message::new_user(norms);

            self.messages.push(message);
        }
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn new_system(content: impl Into<String>) -> Self {
        let role = Role::System;

        Self {
            role,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn new_user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn new_tool(content: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: Some(id.into()),
        }
    }
}

#[derive(Debug, Serialize, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ToolCall {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub id: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}
