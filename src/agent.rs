use colored::Colorize;
use eyre::Context;
use tokio::{spawn, sync::mpsc::{UnboundedSender, unbounded_channel}, task::JoinHandle};

pub struct Agent {
    next: UnboundedSender<String>
}

impl Agent {
    pub fn spawn(next_address: UnboundedSender<String>) -> AgentHandle {
        let (tx, mut rx) = unbounded_channel::<String>();

        let handle = spawn(async move {
            let agent = Agent {next: next_address};

            while let Some(prompt) = rx.recv().await {
                println!("{}", format!("Received {prompt} in agent.").blue());

                agent.next.send("meow".to_owned()).unwrap();
            }
        });

        AgentHandle { tx, handle }
    }
}

pub struct AgentHandle {
    tx: UnboundedSender<String>,
    handle: JoinHandle<()>,
}

impl AgentHandle {
    pub fn send(&self, prompt: String) -> eyre::Result<()> {
        self.tx.send(prompt).context("Sending message to agent")
    }
}
