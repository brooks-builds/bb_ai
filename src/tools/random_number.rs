use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools};
use eyre::{Context, Result};
use serde_json::json;
use tokio::{
    spawn,
    sync::{mpsc, oneshot},
};

use crate::tools::Tool;

pub struct RandomNumber {
    receiver: mpsc::Receiver<RandomNumberMessage>,
}

impl RandomNumber {
    pub fn new(receiver: mpsc::Receiver<RandomNumberMessage>) -> Self {
        Self { receiver }
    }

    pub fn handle_message(&mut self, message: RandomNumberMessage) -> Result<()> {
        let random_number = rand::random_range(message.min..=message.max);

        message.respond_to.send(random_number).unwrap();

        Ok(())
    }

    pub async fn run(mut self) -> Result<()> {
        while let Some(message) = self.receiver.recv().await {
            self.handle_message(message)?;
        }

        Ok(())
    }
}

pub struct RandomNumberMessage {
    respond_to: oneshot::Sender<i32>,
    min: i32,
    max: i32,
}

pub struct RandomNumberHandle {
    sender: mpsc::Sender<RandomNumberMessage>,
}

impl RandomNumberHandle {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(8);
        let actor = RandomNumber::new(rx);

        spawn(actor.run());

        Self { sender: tx }
    }

    pub async fn send_to(&self, min: i32, max: i32) -> Result<i32> {
        let (tx, rx) = oneshot::channel();
        let message = RandomNumberMessage {
            respond_to: tx,
            min,
            max,
        };

        self.sender.send(message).await.unwrap();

        rx.await.context("getting random number tool call result")
    }
}

impl Default for RandomNumberHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for RandomNumberHandle {
    fn definition(&self) -> ChatCompletionTools {
        let description =
            Some("Generate a random integer between min and max (inclusive)".to_owned());
        let parameters = Some(json!({
            "type": "object",
            "properties": {
                "min": {
                    "type": "number",
                    "description": "The lowest possible integer the random number can be"
                },
                "max": {
                    "type": "number",
                    "description": "The highest possible integer (inclusive) that the random number can be"
                }
            },
            "required": ["min", "max"],
            "additionalProperties": false
        }));
        let strict = Some(true);

        ChatCompletionTools::Function(ChatCompletionTool {
            function: async_openai::types::chat::FunctionObject {
                name: self.name(),
                description,
                parameters,
                strict,
            },
        })
    }

    fn name(&self) -> String {
        "random_number".to_owned()
    }
}
