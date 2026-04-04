use std::{
    env,
    path::{Path, PathBuf},
};

use eyre::{Context, Result, bail};
use ignore::Walk;
use serde::Deserialize;
use serde_json::Value;
use tokio::{
    spawn,
    sync::{mpsc, oneshot},
};

struct ListFiles {
    receiver: mpsc::Receiver<ListFilesMessage>,
}

impl ListFiles {
    fn new(receiver: mpsc::Receiver<ListFilesMessage>) -> Self {
        Self { receiver }
    }

    async fn run(mut self) -> Result<()> {
        while let Some(message) = self.receiver.recv().await {
            let path = message
                .path
                .unwrap_or_else(|| env::current_dir().unwrap())
                .canonicalize()?;

            self.guard(&path)?;

            let paths = Walk::new(path)
                .fold(vec![], |mut paths, dir_entry| {
                    if let Ok(entry) = dir_entry {
                        paths.push(entry.path().to_string_lossy().to_string());
                    }

                    paths
                })
                .join("\n");

            message.respond_to.send(paths).unwrap();
        }

        Ok(())
    }

    fn guard(&self, path: &Path) -> Result<()> {
        let current_dir = env::current_dir()?;

        if !path
            .to_string_lossy()
            .contains(&current_dir.to_string_lossy().to_string())
        {
            bail!("Attempting to list files outside of current directory");
        }

        Ok(())
    }
}

struct ListFilesMessage {
    respond_to: oneshot::Sender<String>,
    path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ListFileHandle {
    sender: mpsc::Sender<ListFilesMessage>,
}

impl ListFileHandle {
    pub fn spawn() -> Result<Self> {
        let (tx, rx) = mpsc::channel(8);
        let list_files = ListFiles::new(rx);

        spawn(list_files.run());

        Ok(Self { sender: tx })
    }

    pub async fn send(&self, arguments: &str) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        let args = serde_json::from_str::<ListFileArgs>(arguments)?;
        let message = ListFilesMessage {
            respond_to: tx,
            path: args.path,
        };

        self.sender
            .send(message)
            .await
            .context("Sending command to list file actor")?;
        rx.await.context("getting results from list files actor")
    }

    pub fn name(&self) -> &'static str {
        "list_files"
    }

    pub fn definition(&self) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name(),
                "description": "List files in the provided directory. If none is provided, all files in the current directory will be listed. This is a recursive tool, so all files from children directories will also be returned.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Absolute or relative path to a directory. This must be in the current directory."
                        }
                    }
                }
            }
        })
    }

    pub fn system_prompt(&self) -> String {
        let name = self.name();

        format!(
            "\n{name} is a tool you have access to. It allows you to list files that you have access to. This can be paired with other tools if you have them.\n"
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct ListFileArgs {
    pub path: Option<PathBuf>,
}
