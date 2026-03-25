use crate::{context::Message, tools::BBTool};
use colored::Colorize;
use serde_json::json;
use std::process::Command;

pub const NAME: &str = "git_diff";

pub struct GitDiffTool;

impl BBTool for GitDiffTool {
    type Arguments = ();

    fn run(_args: &str, id: String) -> Result<crate::context::Message, crate::context::Message> {
        println!("{}", "Running git diff".green());

        let output = match Command::new("git").arg("diff").output() {
            Ok(output) => output,
            Err(error) => {
                eprintln!(
                    "{}",
                    format!("Error running git diff command: {error}").red()
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
            "description": "Run the git diff command for all files that will be committed."
          }
        })
    }
}
