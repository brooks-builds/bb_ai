use crate::{AgentResponse, ai_command::BBAiCommand};
use serde_json::Value;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct Config {
    pub user_input: UnboundedReceiver<BBAiCommand>,
    pub response: UnboundedSender<AgentResponse>,
    pub system_prompt: String,
    pub model: String,
    pub api_base_url: String,
    pub api_key: String,
    pub tools: Vec<Value>,
    pub norms: Option<String>,
}
