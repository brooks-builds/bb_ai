use colored::Colorize;
use eyre::Context;
use std::{
    io::{Write, stdin, stdout},
    sync::mpsc::Receiver,
};

pub fn get_prompt(
    file_watcher_rx: &mut Receiver<notify::Result<notify::Event>>,
) -> eyre::Result<Command> {
    let mut changed_paths = vec![];

    while let Ok(Ok(file)) = file_watcher_rx.try_recv() {
        println!(
            "{}",
            format!("{file:?} changed").truecolor(0xFF, 0xB8, 0x6C)
        );

        changed_paths.push(file.paths);
    }

    if !changed_paths.is_empty() {
        let changed_paths = format!("user has changed files at: {changed_paths:?}");
        return Ok(Command::Prompt(changed_paths));
    }

    // let mut result = String::new();

    // print!("> ");
    // stdout()
    //     .flush()
    //     .context("flushing standard out to print prompt")?;

    // stdin()
    //     .read_line(&mut result)
    //     .context("getting user prompt")?;

    // Ok(result.into())
    Ok(Command::Nothing)
}

pub enum Command {
    Prompt(String),
    ResetContext,
    Nothing,
}

impl From<String> for Command {
    fn from(user_input: String) -> Self {
        if !user_input.starts_with("/") {
            return Self::Prompt(user_input);
        }

        let input = match user_input.split_ascii_whitespace().next() {
            Some(input) => input,
            None => return Self::Nothing,
        };

        match input {
            "/reset" => Self::ResetContext,
            "/clear" => Self::ResetContext,
            _ => Self::Nothing,
        }
    }
}
