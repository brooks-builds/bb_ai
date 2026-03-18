use crate::{
    context::Message,
    tools::{BBTool, utilities},
};
use colored::Colorize;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    env,
    fs::OpenOptions,
    io::Read,
    path::{Path, PathBuf},
};

pub const NAME: &str = "read_file";

pub struct ReadFileTool;

impl BBTool for ReadFileTool {
    fn definition() -> Value {
        json!({
          "type": "function",
          "function": {
            "name": NAME,
            "description": "Read the entire contents of the given file. Hidden files are forbidden.",
            "parameters": {
              "type": "object",
              "properties": {
                "path": {
                  "type": "string",
                  "description": "path to the file to read."
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
                return Err(Message::new_tool(
                    format!("Error parsing the arguments: {error}"),
                    id,
                ));
            }
        };
        let project_directory = match env::current_dir() {
            Ok(path) => path,
            Err(error) => return Err(Message::new_tool(format!("Error: {error}"), id)),
        };
        let path = project_directory.join(args.path);

        if is_forbidden(&path) {
            println!(
                "{}",
                format!("Attempted to read illegal file {path:?}").red()
            );
            return Err(Message::new_tool(
                "Error, the provided file is not legal to read.".to_owned(),
                id,
            ));
        }

        println!("{}", format!("Reading file at path: {path:?}").green());

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

fn is_forbidden(path: &Path) -> bool {
    let filename = match path.file_name() {
        Some(name) => name,
        None => return true,
    };
    let project_directory = match env::current_dir() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => return true,
    };

    if !path.to_string_lossy().contains(&project_directory) {
        return true;
    }

    if filename.to_string_lossy().starts_with(".") {
        return true;
    }

    if path.to_string_lossy().contains("..") {
        return true;
    }

    false
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use std::path::Path;

    #[test]
    fn forbidden_to_read_outside_project_directory() -> eyre::Result<()> {
        let read_path = Path::new("/README.md");

        assert!(is_forbidden(read_path));

        Ok(())
    }

    #[test]
    fn forbidden_to_read_file_with_relative_up_in_path() -> eyre::Result<()> {
        let current_directory = env::current_dir()?;
        let path = current_directory.join("../README.md");

        assert!(is_forbidden(&path));
        Ok(())
    }
}
