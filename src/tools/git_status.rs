use std::process::Command;

use eyre::{Context, Result};
use tokio::{
    spawn,
    sync::{mpsc, oneshot},
};

struct GitStatus {
    receiver: mpsc::Receiver<GitStatusMessage>,
}

impl GitStatus {
    async fn run(mut self) -> Result<()> {
        while let Some(message) = self.receiver.recv().await {
            #[allow(irrefutable_let_patterns)]
            if let GitStatusMessage::Run { respond_to } = message {
                let output = Command::new("git")
                    .arg("status")
                    .output()
                    .context("Running git status command")?;
                let git_status = String::from_utf8(output.stdout)
                    .context("Converting command output to string")?;

                respond_to
                    .send(git_status)
                    .expect("Sending git status command output");
            }
        }
        Ok(())
    }
}

enum GitStatusMessage {
    Run { respond_to: oneshot::Sender<String> },
}

#[derive(Debug, Clone)]
pub struct GitStatusHandle {
    sender: mpsc::Sender<GitStatusMessage>,
}

impl GitStatusHandle {
    pub fn spawn() -> Result<Self> {
        let (tx, rx) = mpsc::channel(8);
        let git_status = GitStatus { receiver: rx };

        spawn(git_status.run());

        Ok(Self { sender: tx })
    }

    pub async fn send(&self) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        let message = GitStatusMessage::Run { respond_to: tx };

        self.sender
            .send(message)
            .await
            .context("Sending command to Git Status Tool")?;

        rx.await.context("Getting git status tool response")
    }

    pub fn definition(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "git_status",
                "description": "Run the command git status to see the results of the git status command."
            }
        })
    }
}
