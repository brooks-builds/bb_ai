use eyre::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use tokio::{
    spawn,
    sync::{mpsc, oneshot},
};

use crate::{agent::AgentHandle, tools::ToolMessage};

struct SpawnAgent {
    receiver: mpsc::Receiver<SpawnAgentMessage>,
}

impl SpawnAgent {
    fn new(receiver: mpsc::Receiver<SpawnAgentMessage>) -> Self {
        Self { receiver }
    }

    async fn run(mut self) -> Result<()> {
        while let Some(SpawnAgentMessage {
            respond_to,
            api_base,
            api_key,
            model,
            tool_tx,
            tools,
            system_prompt,
            prompt,
        }) = self.receiver.recv().await
        {
            let agent_handle =
                AgentHandle::spawn(api_base, api_key, model, tool_tx, tools, system_prompt);

            let mut response_rx = agent_handle
                .send(prompt)
                .await
                .context("Sending prompt to sub agent")?;

            let mut results = vec![];

            loop {
                let Some(agent_response) = response_rx.recv().await else {
                    break;
                };

                if !agent_response.content.is_empty() {
                    results.push(agent_response.content);
                }

                if agent_response.finished {
                    break;
                }
            }

            respond_to.send(results.join("\n")).unwrap();
        }

        Ok(())
    }
}

struct SpawnAgentMessage {
    respond_to: oneshot::Sender<String>,
    api_base: String,
    api_key: String,
    model: String,
    tool_tx: Option<mpsc::Sender<ToolMessage>>,
    tools: Vec<Value>,
    system_prompt: String,
    prompt: String,
}

#[derive(Debug, Clone)]
pub struct SpawnAgentHandle {
    sender: mpsc::Sender<SpawnAgentMessage>,
}

impl SpawnAgentHandle {
    pub fn spawn() -> Result<Self> {
        let (tx, rx) = mpsc::channel(8);
        let spawn_agent = SpawnAgent::new(rx);

        spawn(spawn_agent.run());

        Ok(Self { sender: tx })
    }

    pub async fn send(
        &self,
        api_base: String,
        api_key: String,
        model: String,
        tool_tx: Option<mpsc::Sender<ToolMessage>>,
        tools: Vec<Value>,
        system_prompt: String,
        prompt: String,
    ) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        let message = SpawnAgentMessage {
            respond_to: tx,
            api_base,
            api_key,
            model,
            tool_tx,
            tools,
            system_prompt,
            prompt,
        };

        self.sender
            .send(message)
            .await
            .context("Sending message to spawn agent tool")?;

        rx.await.context("Getting responce from sub agent tool")
    }

    pub fn name(&self) -> &'static str {
        "spawn_agent"
    }

    pub fn definition(&self) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name(),
                "description": "Spawn a LLM based agent to complete a task. It will be able to use any tools assigned to it.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "tools": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "List of tool names to give to the agent so that it can do it's job."
                        },
                        "prompt": {
                            "type": "string",
                            "description": "The prompt to tell the sub agent what to do."
                        }
                    },
                    "required": ["tools", "prompt"]
                }
            }
        })
    }

    pub fn system_prompt(&self) -> String {
        let name = self.name();

        format!(
            "\n{name} is a tool you have access to. It allows you to spawn a sub agent LLM that will have access to whatever tools you pass in. The tool list is the same as the tools you have access to, be sure to use the tool name correctly when calling this tool.\n"
        )
    }

    pub fn parse_args(&self, args: &str) -> Result<SpawnAgentArgs> {
        serde_json::from_str(args).context("Parsing arguments from tool call")
    }
}

#[derive(Debug, Deserialize)]
pub struct SpawnAgentArgs {
    pub tools: Vec<String>,
    pub prompt: String,
}
