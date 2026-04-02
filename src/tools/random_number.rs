use eyre::{Context, Result};
use rand::random_range;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::{
    spawn,
    sync::{mpsc, oneshot},
};

struct RandomNumber {
    receiver: mpsc::Receiver<RandomNumberMessage>,
}

impl RandomNumber {
    pub fn new(receiver: mpsc::Receiver<RandomNumberMessage>) -> Self {
        Self { receiver }
    }

    async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            let result = random_range(message.min..=message.max);

            message.respond_to.send(result.to_string()).unwrap();
        }
    }
}

struct RandomNumberMessage {
    respond_to: oneshot::Sender<String>,
    min: i32,
    max: i32,
}

pub struct RandomNumberHandle {
    sender: mpsc::Sender<RandomNumberMessage>,
}

impl RandomNumberHandle {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(1);
        let random_number = RandomNumber::new(rx);

        spawn(random_number.run());

        Self { sender: tx }
    }

    pub async fn send(&self, args: &str) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        let args = serde_json::from_str::<RandomNumberArgs>(args)?;
        let message = RandomNumberMessage {
            respond_to: tx,
            min: args.min,
            max: args.max,
        };

        self.sender.send(message).await?;

        rx.await.context("Getting random number result")
    }

    pub fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "random_number",
                "description": "generate a random interger between min and max (inclusive)",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "min": {
                            "type": "number",
                            "description": "Lowest integer in the range of possible numbers."
                        },
                        "max": {
                            "type": "number",
                            "description": "Largest possible integer in the range of possible numbers, this is inclusive to the range."
                        }
                    },
                    "required": ["min", "max"]
                }
            }
        })
    }
}

#[derive(Deserialize)]
struct RandomNumberArgs {
    min: i32,
    max: i32,
}
