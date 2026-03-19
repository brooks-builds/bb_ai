use crate::{
    context::Message,
    tools::{BBTool, utilities::get_current_directory},
};
use colored::Colorize;
use serde::Deserialize;
use serde_json::json;
use std::{
    env,
    fs::OpenOptions,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

pub const NAME: &str = "append_to_file";

pub struct AppendToFileTool;

impl BBTool for AppendToFileTool {
    type Arguments = Arguments;

    fn run(args: &str, id: String) -> Result<crate::context::Message, crate::context::Message> {
        let arguments = match serde_json::from_str::<Arguments>(args) {
            Ok(args) => args,
            Err(error) => {
                eprintln!("{error:?}");
                return Err(Message::new_tool(
                    format!("Error reading arguments: {error:?}"),
                    id,
                ));
            }
        };

        let path = if arguments.path.is_absolute() {
            arguments.path
        } else {
            match arguments.path.canonicalize() {
                Ok(path) => path,
                Err(error) => {
                    let kind = error.kind();
                    if matches!(kind, ErrorKind::NotFound) {
                        let current_directory = get_current_directory(id.clone())?;
                        current_directory.join(arguments.path)
                    } else {
                        eprintln!("{}", format!("Error {error:?}").red());
                        return Err(Message::new_tool(format!("Error: {error:?}"), id));
                    }
                }
            }
        };

        if !guard(&path) {
            return Err(Message::new_tool(
                "error, the path you passed in is illegal",
                id,
            ));
        }

        println!("{}", format!("Appending to {path:?}").green());

        let mut file = match OpenOptions::new().create(true).append(true).open(path) {
            Ok(file) => file,
            Err(error) => {
                eprintln!(
                    "{}",
                    format!("Error opening file to append to: {error:?}").red()
                );
                return Err(Message::new_tool(
                    format!("Error opening file for appending: {error:?}"),
                    id,
                ));
            }
        };

        if let Err(error) = file.write_all(arguments.content.as_bytes()) {
            eprintln!(
                "{}",
                format!("Error appending to file handler: {error:?}").red()
            );
            return Err(Message::new_tool(
                format!("error appending to file: {error:?}"),
                id,
            ));
        }

        Ok(Message::new_tool("Write success", id))
    }

    fn definition() -> serde_json::Value {
        json!({
          "type": "function",
          "function": {
            "name": NAME,
            "description": "Append a string to a file. This will create the file if it doesn't exist. All data in the file remains.",
            "parameters": {
              "type": "object",
              "properties": {
                "path": {
                  "type": "string",
                  "description": "The path to the file to append to."
                },
                "content": {
                    "type": "string",
                    "description": "The content to append to the file."
                }
              },
              "required": ["path", "content"]
            }
          }
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Arguments {
    path: PathBuf,
    content: String,
}

fn guard(path: &Path) -> bool {
    let current_directory = match env::current_dir() {
        Ok(path) => path,
        Err(_) => return false,
    };

    if !path
        .to_string_lossy()
        .contains(&current_directory.to_string_lossy().to_string())
    {
        eprintln!("{}", format!("Denied append to file at {path:?}").red());
        return false;
    }

    if path.is_dir() {
        eprintln!("{}", "denied appending to a directory".red());
        return false;
    }

    true
}
