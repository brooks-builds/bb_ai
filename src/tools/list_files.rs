use crate::context::Message;
use ignore::Walk;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    env,
    path::{Path, PathBuf},
};

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

pub fn run_tool(arguments: &str, id: impl Into<String>) -> Message {
    dbg!("running tool", TOOL_NAME, &arguments);

    let args = match serde_json::from_str::<ListFilesArgs>(arguments) {
        Ok(args) => args,
        Err(error) => {
            dbg!(&error);
            return Message::new_tool(
                format!("Error, the arguments passed in was invalid: {error}"),
                id,
            );
        }
    };
    let project_root = match env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            dbg!(error);
            return Message::new_tool(
                "Error, this project doesn't seem to have a current directory",
                id,
            );
        }
    };
    let mut content = vec![];
    let path = match args.path.strip_prefix("/") {
        Ok(path) => path,
        Err(error) => {
            dbg!(error);
            Path::new(".")
        }
    };
    let target_directory = project_root.join(path);

    dbg!(&project_root, &target_directory);

    for entry in Walk::new(target_directory) {
        let Ok(entry) = entry else {
            continue;
        };
        dbg!(&entry);
        let path = entry.path().to_string_lossy();

        content.push(format!("path: {path}"));
    }

    dbg!(&content);

    Message::new_tool(content.join("\n"), id)
}

#[derive(Debug, Deserialize)]
pub struct ListFilesArgs {
    pub path: PathBuf,
}
