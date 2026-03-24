use colored::Colorize;
use rand::{RngExt, rng};
use serde::Deserialize;
use serde_json::json;

use crate::{context::Message, tools::BBTool};

pub const NAME: &str = "random_number";

pub struct RandomNumberTool;

impl BBTool for RandomNumberTool {
    type Arguments = RandomNumberArgs;

    fn definition() -> serde_json::Value {
        json!({
          "type": "function",
          "function": {
            "name": NAME,
            "description": "Generate a random integer between the given min and max (inclusive) arguments. For example you can generate a number between 1 and 100.",
            "parameters": {
              "type": "object",
              "properties": {
                "min": {
                  "type": "number",
                  "description": "the lowest integer that can be randomly generated. For example 1"
                },
                "max": {
                    "type": "number",
                    "description": "the highest possible integer that can be randomly generated. For example 100"
                }
              },
              "required": ["min", "max"]
            }
          }
        })
    }

    fn run(args: &str, id: String) -> Result<crate::context::Message, crate::context::Message> {
        println!("{}", "Running random number generator tool".green());
        let args = match serde_json::from_str::<Self::Arguments>(args) {
            Ok(args) => args,
            Err(error) => {
                eprintln!("{}", format!("{error:?}").red());
                return Err(Message::new_tool(format!("Error; {error:?}"), id));
            }
        };
        let result = rng().random_range(args.min..=args.max);
        println!(
            "{}",
            format!("Random number tool results with {result}").green()
        );

        Ok(Message::new_tool(format!("{result}"), id))
    }
}

#[derive(Debug, Deserialize)]
pub struct RandomNumberArgs {
    pub min: i32,
    pub max: i32,
}
