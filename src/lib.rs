use serde_json::Value;

pub mod agent;
pub mod llm_sender;

pub enum AppMessage {
    AgentIO {
        content: String,
        cost: f32,
        tokens_used: u32,
    },
    LlmSenderIO(Value),
}
