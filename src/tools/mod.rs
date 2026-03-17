pub mod utilities;
pub mod list_files;
pub mod read_file;
pub mod request_tool;

use serde_json::Value;
use crate::context::Message;

pub trait BBTool {
    fn tool_name() -> &'static str;
    fn definition() -> Value;
    fn run(args: String, id) -> Message;
}
