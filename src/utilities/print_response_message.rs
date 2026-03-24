use crate::AgentResponse;

pub fn print_ai_response(response: &AgentResponse) {
    let context_length = response.context_length;
    let cost = response.cost;

    if let Some(message) = &response.message {
        println!("Context: {context_length}, Cost: {cost}::{message}");
    }
}
