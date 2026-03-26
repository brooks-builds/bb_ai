pub mod append_to_file;
pub mod git_diff;
pub mod git_status;
pub mod list_files;
pub mod random_number;
pub mod read_file;
pub mod request_tool;
pub mod sub_agent;
pub mod utilities;

use crate::context::Message;
use serde_json::Value;

pub trait BBTool {
    type Arguments;

    fn definition() -> Value;
    fn run(&mut self, args: &str, id: String) -> Result<Message, Message>;
}

pub fn run_tools(arguments: &str, id: String, name: &str) -> Result<Message, Message> {
    match name {
        list_files::TOOL_NAME => list_files::run_tool(arguments, id),
        read_file::NAME => read_file::ReadFileTool::run(arguments, id),
        append_to_file::NAME => append_to_file::AppendToFileTool::run(arguments, id),
        random_number::NAME => random_number::RandomNumberTool::run(arguments, id),
        git_diff::NAME => git_diff::GitDiffTool::run(arguments, id),
        git_status::NAME => git_status::GitStatusTool::run(arguments, id),
        _ => Err(Message::new_tool(
            format!("Error, tool {name} doesn't exist."),
            id,
        )),
    }
}
