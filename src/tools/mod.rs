pub mod append_to_file;
pub mod list_files;
pub mod read_file;
pub mod request_tool;
pub mod utilities;

use crate::context::Message;
use serde_json::Value;

pub trait BBTool {
    type Arguments;

    fn definition() -> Value;
    fn run(args: &str, id: String) -> Result<Message, Message>;
}

pub fn run_tools(arguments: &str, id: String, name: &str) -> Result<Message, Message> {
    match name {
        list_files::TOOL_NAME => list_files::run_tool(arguments, id),
        read_file::NAME => read_file::ReadFileTool::run(arguments, id),
        append_to_file::NAME => append_to_file::AppendToFileTool::run(arguments, id),
        _ => Err(Message::new_tool(
            format!("Error, tool {name} doesn't exist."),
            id,
        )),
    }
}
