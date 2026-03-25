use crate::{context::Message, tools::BBTool};
use colored::Colorize;
use serde_json::json;
use std::process::Command;

pub const NAME: &str = "git_status";

pub struct GitStatusTool;

impl BBTool for GitStatusTool {
    type Arguments = ();

    fn run(_args: &str, id: String) -> Result<crate::context::Message, crate::context::Message> {
        println!("{}", "Running git status".green());

        let output = match Command::new("git").arg("status").output() {
            Ok(output) => output,
            Err(error) => {
                eprintln!(
                    "{}",
                    format!("Error running git status command: {error}").red()
                );
                return Err(Message::new_tool(format!("Error: {error:?}"), id));
            }
        };

        match String::from_utf8(output.stdout) {
            Ok(diff) => Ok(Message::new_tool(diff, id)),
            Err(error) => Err(Message::new_tool(format!("Error: {error:?}"), id)),
        }
    }

    fn definition() -> serde_json::Value {
        json!({
          "type": "function",
          "function": {
            "name": NAME,
            "description": "Run the git status command to get a full list of files that need to be committed. This includes file that are modified (which the git diff command will help see the differences) and new files that haven't been committed yet."
          }
        })
    }
}
