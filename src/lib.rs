use serde_json::Value;

pub mod agent;
pub mod llm_sender;

pub enum AppMessage {
    AgentIO {
        content: String,
        cost: Option<f32>,
        tokens_used: Option<u32>,
    },
    LlmSenderIO(Value),
}
