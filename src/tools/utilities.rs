use crate::context::Message;
use std::{env, path::PathBuf};

pub fn get_current_directory(id: String) -> Result<PathBuf, Message> {
    let project_root = match env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("{error:?}");
            return Err(Message::new_tool(
                "Error, this project doesn't seem to have a current directory",
                id,
            ));
        }
    };

    Ok(project_root)
}
