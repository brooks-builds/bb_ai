use std::path::PathBuf;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::{context::Message, tools::BBTool};

pub struct ReadFileTool;

impl BBTool for ReadFileTool {
    fn tool_name() -> &'static str {
        "read_file"
    }

    fn definition() -> Value {
        json!({
          "type": "function",
          "function": {
            "name": Self::tool_name(),
            "description": "Read the entire contents of the given file.",
            "parameters": {
              "type": "object",
              "properties": {
                "path": {
                  "type": "string",
                  "description": "Path to the file to read."
                }
              },
              "required": ["path"]
            }
          }
        })
    }

    fn run(args: String, id: String) -> crate::context::Message {
        let args = match serde_json::from_str::<Args>(&args) {
            Ok(args) => args,
            Err(error) => {
                eprintln!("{error:?}");
                return Message::new_tool(format!("Error parsing the arguments: {error}"), id);
            }
        };
    }
}

#[derive(Debug, Deserialize)]
struct Args {
    pub path: PathBuf,
}
