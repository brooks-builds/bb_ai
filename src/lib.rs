use serde_json::Value;

pub mod agent;
pub mod llm_sender;

pub enum AppMessage {
    AgentIn(String),
    AgentOut {
        content: String,
        finished: bool,
    },
    LlmSenderIO(Value),
}
