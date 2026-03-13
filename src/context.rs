use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct ChatContext {
    pub model: String,
    pub messages: Vec<Message>,
    pub tools: Vec<Value>,
}

impl ChatContext {
    pub fn new(
        model: impl Into<String>,
        system_prompt: impl Into<String>,
        tools: Vec<Value>,
    ) -> Self {
        let messages = vec![Message::new_system(system_prompt)];

        Self {
            model: model.into(),
            messages,
            tools,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn new_system(content: impl Into<String>) -> Self {
        let role = Role::System;

        Self {
            role,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn new_user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn new_tool(content: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
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
