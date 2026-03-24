use std::io::{Write, stdin, stdout};

pub fn get_user_input() -> eyre::Result<String> {
    print!("> ");
    stdout().flush()?;

    let mut prompt = String::new();
    stdin().read_line(&mut prompt)?;

    Ok(prompt)
}
