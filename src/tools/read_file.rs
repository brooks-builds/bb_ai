use std::{
    env,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use eyre::{Context, Result, bail};
use serde::Deserialize;
use serde_json::Value;
use tokio::{
    spawn,
    sync::{mpsc, oneshot},
};

struct ReadFile {
    receiver: mpsc::Receiver<ReadFileMessage>,
}

impl ReadFile {
    fn new(receiver: mpsc::Receiver<ReadFileMessage>) -> Self {
        Self { receiver }
    }

    async fn run(mut self) -> Result<()> {
        while let Some(message) = self.receiver.recv().await {
            let path = if message.path.is_absolute() {
                message.path
            } else {
                let current_dir =
                    env::current_dir().context("Getting current working directory")?;

                current_dir.join(message.path)
            }
            .canonicalize()
            .context("Cananicalizing the path")?;

            self.guard(&path)?;

            let content = read_to_string(path).context("Reading file to string")?;

            message.respond_to.send(content).unwrap();
        }

        Ok(())
    }

    fn guard(&self, path: &Path) -> Result<()> {
        let current_dir = env::current_dir().context("Getting current working directory")?;

        if !path
            .to_string_lossy()
            .contains(&current_dir.to_string_lossy().to_string())
        {
            bail!("Attempting to read file outside of current directory");
        }

        let Some(filename) = path.file_name() else {
            bail!("File name must be part of path");
        };

        if filename == ".env" {
            bail!(".env is protected and cannot be read");
        }

        Ok(())
    }
}

struct ReadFileMessage {
    respond_to: oneshot::Sender<String>,
    path: PathBuf,
}

pub struct ReadFileHandle {
    sender: mpsc::Sender<ReadFileMessage>,
}

impl ReadFileHandle {
    pub fn spawn() -> Result<Self> {
        let (tx, rx) = mpsc::channel(8);
        let read_file = ReadFile::new(rx);

        spawn(read_file.run());

        Ok(Self { sender: tx })
    }

    pub async fn send(&self, arguments: String) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        let path = serde_json::from_str::<Args>(&arguments).context("Converting arguments")?;
        let message = ReadFileMessage {
            respond_to: tx,
            path: path.path,
        };

        self.sender.send(message).await.context("Sending command")?;

        rx.await.context("Getting result from read file tool")
    }

    pub fn name(&self) -> &'static str {
        "read_file"
    }

    pub fn definition(&self) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name(),
                "description": "Read a file from disk given a path. The path must be in the current directory. And cannot be a .env file. It can be relative or absolute. For example you can attempt to read `/User/username/code/project/src/main.rs` or `./main.rs`",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute or relative path to a file you want to read. This must be in the current directory, and cannot be a .env file."
                        }
                    },
                    "required": ["path"]
                }
            }
        })
    }

    pub fn system_prompt(&self) -> String {
        let name = self.name();

        format!(
            "\n{name} is a tool you have access to. It allows you to read a file in the current directory. You cannot use it to read files outside the current directory, nor can you read secret files like .env. Use this tool whenever you want to see the contents of a file.\n"
        )
    }
}

#[derive(Debug, Deserialize)]
struct Args {
    path: PathBuf,
}
