use std::io::{Write, stdin, stdout};

use eyre::Context;

pub fn get_prompt() -> eyre::Result<Command> {
    let mut result = String::new();

    print!("> ");
    stdout()
        .flush()
        .context("flushing standard out to print prompt")?;

    stdin()
        .read_line(&mut result)
        .context("getting user prompt")?;

    Ok(result.into())
}

pub enum Command {
    Prompt(String),
    ResetContext,
    Nothing,
}

impl From<String> for Command {
    fn from(user_input: String) -> Self {
        if !user_input.starts_with("/") {
            return Self::Prompt(user_input)
        }

        let input = match user_input.split_ascii_whitespace().next() {
            Some(input) => input,
            None => return Self::Nothing,
        };

        match input {
            "/reset" => Self::ResetContext,
            "/clear" => Self::ResetContext,
            _ => Self::Nothing
        }
    }
}
