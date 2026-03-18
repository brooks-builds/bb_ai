use crate::{context::Message, tools::utilities::get_current_directory};
use colored::Colorize;
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
        "description": "List all files and directories given a relative or absolute path. Relative paths start with the current directory.",
        "parameters": {
          "type": "object",
          "properties": {
            "path": {
              "type": "string",
              "description": "Path to get a list of files and directories in. This can be absolute or relative like ."
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
            return Err(Message::new_tool(
                format!("Error, the arguments passed in was invalid: {error}"),
                id,
            ));
        }
    };
    let path = get_current_directory(id.clone())?;
    let mut content = vec![];
    let target_directory = if args.path.is_relative() {
        path.join(args.path)
    } else {
        args.path
    };

    if is_denied(&target_directory) {
        println!(
            "{}",
            format!(
                "Attempted to read directories outside current directory at: {target_directory:?}"
            )
            .red()
        );
        return Err(Message::new_tool(
            "Error, you are not allowed to search outside the current directory",
            id,
        ));
    }

    println!(
        "{}",
        format!("running tool list files at path: {target_directory:?}").green()
    );

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

fn is_denied(path: &Path) -> bool {
    let project_directory = match env::current_dir() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => return true,
    };

    if !path.to_string_lossy().contains(&project_directory) {
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

    #[test]
    fn allowed_to_search_same_directory() -> eyre::Result<()> {
        let current_directory = std::env::current_dir()?;

        assert!(!is_denied(&current_directory));

        Ok(())
    }

    #[test]
    fn allowed_to_search_same_relative_directory() -> eyre::Result<()> {
        let current_directory = std::env::current_dir()?;
        let path = current_directory.join(".");

        assert!(!is_denied(&path));

        Ok(())
    }

    #[test]
    fn denied_searching_outside_project_directory() -> eyre::Result<()> {
        let path = Path::new("/");

        assert!(is_denied(path));

        Ok(())
    }

    #[test]
    fn denied_searching_outside_project_directory_with_relative_path() -> eyre::Result<()> {
        let path = std::env::current_dir()?.join("..");

        assert!(is_denied(&path));

        Ok(())
    }
}
