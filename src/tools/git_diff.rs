use std::process::Command;

use eyre::{Context, Result};
use serde_json::{Value, json};
use tokio::{
    spawn,
    sync::{mpsc, oneshot},
};

struct GitDiff {
    receiver: mpsc::Receiver<GitDiffMessage>,
}

impl GitDiff {
    fn new(receiver: mpsc::Receiver<GitDiffMessage>) -> Self {
        Self { receiver }
    }

    async fn run(mut self) -> Result<()> {
        while let Some(message) = self.receiver.recv().await {
            let git_diff_command = Command::new("git").arg("diff").output()?;
            let git_diff = String::from_utf8(git_diff_command.stdout)?;

            message
                .respond_to
                .send(git_diff)
                .expect("Sending git diff result");
        }

        Ok(())
    }
}

struct GitDiffMessage {
    respond_to: oneshot::Sender<String>,
}

#[derive(Debug, Clone)]
pub struct GitDiffHandle {
    sender: mpsc::Sender<GitDiffMessage>,
}

impl GitDiffHandle {
    pub fn spawn() -> Result<Self> {
        let (tx, rx) = mpsc::channel(1);
        let git_diff = GitDiff::new(rx);

        spawn(git_diff.run());

        Ok(Self { sender: tx })
    }

    pub async fn send(&self) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        let message = GitDiffMessage { respond_to: tx };

        self.sender
            .send(message)
            .await
            .context("Sending request to run git diff")?;

        rx.await.context("Getting result of git diff")
    }

    pub fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "git_diff",
                "description": "Run the command git diff to see the changes in the current repo."
            }
        })
    }
}
