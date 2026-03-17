use crate::{context::Message, tools::utilities::get_current_directory};
use ignore::Walk;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub const TOOL_NAME: &str = "list_files";

pub fn tool_definition() -> Value {
    json!({
      "type": "function",
      "function": {
        "name": TOOL_NAME,
        "description": "List all files and directories within the given path.",
        "parameters": {
          "type": "object",
          "properties": {
            "path": {
              "type": "string",
              "description": "Path to get a list of files and directories in. / is the root of the project."
            }
          },
          "required": ["path"]
        }
      }
    })
}

pub fn run_tool(arguments: &str, id: String) -> Result<Message, Message> {
    let args = match serde_json::from_str::<ListFilesArgs>(arguments) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error:?}");
            return Err(Message::new_tool(
                format!("Error, the arguments passed in was invalid: {error}"),
                id,
            ));
        }
    };
    let project_root = get_current_directory(id.clone())?;
    let mut content = vec![];
    let path = match args.path.strip_prefix("/") {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{error:?}");
            Path::new(".")
        }
    };
    let target_directory = project_root.join(path);

    println!("running tool list files at path: {path:?}");

    for entry in Walk::new(target_directory) {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path().to_string_lossy();

        content.push(format!("path: {path}"));
    }

    Ok(Message::new_tool(content.join("\n"), id))
}

#[derive(Debug, Deserialize)]
pub struct ListFilesArgs {
    pub path: PathBuf,
}
