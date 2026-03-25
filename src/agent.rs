use crate::{AgentResponse, ai_command::BBAiCommand, config::Config, context::ChatContext};
use colored::Colorize;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub struct BBAgent {
    chat_context: ChatContext,
    input_rx: UnboundedReceiver<BBAiCommand>,
    system_prompt: String,
    sub_agent_channels: Vec<SubAgentChannels>,
}

impl BBAgent {
    pub fn new(config: Config, sub_agent_channels: Vec<SubAgentChannels>) -> Self {
        let system_prompt = config.system_prompt.clone();
        let chat_context = ChatContext::new(&config);
        let input_rx = config.user_input;

        Self {
            chat_context,
            input_rx,
            system_prompt,
            sub_agent_channels,
        }
    }

    pub async fn run(&mut self) {
        while let Some(command) = self.input_rx.recv().await {
            match command {
                BBAiCommand::Prompt(prompt) => {
                    dbg!(prompt);
                }
                BBAiCommand::ResetContext => {
                    println!("{}", "Resetting context".blue());
                    self.chat_context.reset(self.system_prompt.clone());
                }
            }
        }
    }
}

pub async fn run_agent(mut agents: Vec<BBAgent>) {
    while let Some(mut agent) = agents.pop() {
        tokio::task::spawn(async move {
            agent.run().await;
        });
    }
}

#[derive(Debug)]
pub struct SubAgentChannels {
    pub input_tx: UnboundedSender<BBAiCommand>,
    pub response_rx: UnboundedReceiver<AgentResponse>,
    pub description: String,
}

impl SubAgentChannels {
    pub fn new(
        input_tx: UnboundedSender<BBAiCommand>,
        response_rx: UnboundedReceiver<AgentResponse>,
        description: String,
    ) -> Self {
        Self {
            input_tx,
            response_rx,
            description,
        }
    }
}
