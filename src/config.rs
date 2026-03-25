use crate::{AgentResponse, ai_command::BBAiCommand};
use serde_json::Value;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Debug)]
pub struct Config {
    pub user_input: UnboundedReceiver<BBAiCommand>,
    pub response: UnboundedSender<AgentResponse>,
    pub system_prompt: String,
    pub model: String,
    pub api_base_url: String,
    pub api_key: String,
    pub tools: Vec<Value>,
    pub user_input_tx: Option<UnboundedSender<BBAiCommand>>,
    pub response_rx: Option<UnboundedReceiver<AgentResponse>>,
    pub agents: Vec<Config>,
    pub description: String,
}

impl Config {
    pub fn new(
        system_prompt: impl Into<String>,
        model: impl Into<String>,
        api_base_url: impl Into<String>,
        api_key: impl Into<String>,
        tools: Vec<Value>,
        agents: Vec<Config>,
        description: impl Into<String>,
    ) -> Self {
        let (user_input_tx, user_input_rx) = unbounded_channel();
        let (agent_response_tx, agent_response_rx) = unbounded_channel();

        Self {
            user_input: user_input_rx,
            response: agent_response_tx,
            system_prompt: system_prompt.into(),
            model: model.into(),
            api_base_url: api_base_url.into(),
            api_key: api_key.into(),
            tools,
            user_input_tx: Some(user_input_tx),
            response_rx: Some(agent_response_rx),
            agents,
            description: description.into(),
        }
    }
}
