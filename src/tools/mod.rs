pub mod random_number;

use serde_json::Value;

pub struct ToolMessage {
    id: String,
    name: String,
    arguments: Value,
}
