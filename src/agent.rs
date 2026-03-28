use std::fmt::Display;

use crate::{AppMessage, llm_sender::spawn_llm_sender};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
    spawn,
    sync::mpsc::{UnboundedSender, unbounded_channel},
};

pub async fn spawn_agent(
    respond_to: UnboundedSender<AppMessage>,
    system_prompt: String,
    model: String,
    api_key: String,
    api_base: String,
    stream: bool,
) -> UnboundedSender<AppMessage> {
    let (tx, mut rx) = unbounded_channel();

    {
        let tx = tx.clone();
        spawn(async move {
            println!("{}", "spawning agent actor".red());
            let mut agent = Agent::new(model, system_prompt);
            let llm_sender = spawn_llm_sender(tx.clone(), &api_key, &api_base, stream).await;

            while let Some(message) = rx.recv().await {
                match message {
                    AppMessage::AgentIO { content, .. } => {
                        agent.add_user_message(content);

                        let value = agent.to_value();

                        llm_sender.send(AppMessage::LlmSenderIO(value)).unwrap();
                    }
                    AppMessage::LlmSenderIO(value) => {
                        println!("{}", "receiving value from llm sender actor".red());
                        if stream {
                            dbg!(value);
                        } else {
                            let response = serde_json::from_value::<LlmResponse>(value).unwrap();
                            let message = &response.choices[0].message;
                            let content = format!("{message}");
                            let cost = response.usage.cost;
                            let tokens_used = response.usage.total_tokens;

                            agent.context.add_message(message.clone());
                            agent.cost += cost;
                            agent.tokens_used += tokens_used;

                            let app_message = AppMessage::AgentIO {
                                content,
                                cost: Some(agent.cost),
                                tokens_used: Some(agent.tokens_used),
                            };

                            respond_to.send(app_message).unwrap();
                            
                        }
                    }
                }
            }
        });
    }

    tx
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    pub context: Context,
    #[serde(skip)]
    pub cost: f32,
    #[serde(skip)]
    pub tokens_used: u32,
}

impl Agent {
    pub fn new(model: String, system_prompt: String) -> Self {
        let messages = vec![Message::new_system(system_prompt)];
        let context = Context { model, messages };
        let cost = 0.0;
        let tokens_used = 0;

        Self {
            context,
            cost,
            tokens_used,
        }
    }

    pub fn add_user_message(&mut self, prompt: String) {
        let message = Message::new_user(prompt);

        self.context.add_message(message);
    }

    pub fn to_value(&self) -> Value {
        serde_json::to_value(&self.context).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Context {
    pub model: String,
    pub messages: Vec<Message>,
}

impl Context {
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn new_system(content: String) -> Self {
        let role = Role::System;

        Self { role, content }
    }

    pub fn new_user(content: String) -> Self {
        let role = Role::User;

        Self { role, content }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.role, self.content)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Role::System => "System",
                Role::User => "User",
                Role::Assistant => "Assistant",
                Role::Tool => "Tool",
            }
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct LlmResponse {
    pub choices: Vec<LlmResponseChoice>,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct LlmResponseChoice {
    pub message: Message,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub cost: f32,
    pub total_tokens: u32,
}
