use crate::{
    context::Message,
    tools::{BBTool, utilities},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{fs::OpenOptions, io::Read, path::PathBuf};

pub const NAME: &str = "read_file";

pub struct ReadFileTool;

impl BBTool for ReadFileTool {
    fn definition() -> Value {
        json!({
          "type": "function",
          "function": {
            "name": NAME,
            "description": "Read the entire contents of the given file.",
            "parameters": {
              "type": "object",
              "properties": {
                "path": {
                  "type": "string",
                  "description": "Relative path to the file to read."
                }
              },
              "required": ["path"]
            }
          }
        })
    }

    fn run(args: &str, id: String) -> Result<crate::context::Message, Message> {
        let args = match serde_json::from_str::<Args>(args) {
            Ok(args) => args,
            Err(error) => {
                eprintln!("{error:?}");
                return Err(Message::new_tool(
                    format!("Error parsing the arguments: {error}"),
                    id,
                ));
            }
        };
        let current_directory = utilities::get_current_directory(id.clone())?;
        let args_path = if args.path.has_root() {
            match args.path.strip_prefix("/") {
                Ok(path) => path.to_path_buf(),
                Err(error) => {
                    return Err(Message::new_tool(
                        format!(
                            "There was an interal error getting the path from arguments: {error}"
                        ),
                        id,
                    ));
                }
            }
        } else {
            args.path
        };
        let path = current_directory.join(args_path);
        println!("Reading file at path: {path:?}");
        let mut file = match OpenOptions::new().read(true).open(path) {
            Ok(file) => file,
            Err(error) => {
                return Err(Message::new_tool(
                    format!("Error opening file to read: {error}"),
                    id,
                ));
            }
        };
        let mut content = String::new();

        if let Err(error) = file.read_to_string(&mut content) {
            return Err(Message::new_tool(
                format!("error reading file: {error}"),
                id,
            ));
        }

        Ok(Message::new_tool(content, id))
    }
}

#[derive(Debug, Deserialize)]
struct Args {
    pub path: PathBuf,
}
