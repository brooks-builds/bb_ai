use eyre::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::process::Command;
use tokio::{
    spawn,
    sync::{mpsc, oneshot},
};

struct Say {
    receiver: mpsc::Receiver<SayMessage>,
}

impl Say {
    fn new(receiver: mpsc::Receiver<SayMessage>) -> Self {
        Self { receiver }
    }

    async fn run(mut self) -> Result<()> {
        while let Some(message) = self.receiver.recv().await {
            Command::new("say")
                .arg(message.content)
                .arg("--voice")
                .arg(message.voice)
                .output()?;
            message.respond_to.send("".to_owned()).unwrap();
        }

        Ok(())
    }
}

struct SayMessage {
    respond_to: oneshot::Sender<String>,
    content: String,
    voice: String,
}

#[derive(Debug, Clone)]
pub struct SayHandle {
    sender: mpsc::Sender<SayMessage>,
}

impl SayHandle {
    pub fn spawn() -> Result<Self> {
        let (tx, rx) = mpsc::channel(8);
        let say = Say::new(rx);

        spawn(say.run());

        Ok(Self { sender: tx })
    }

    pub async fn send(&self, arguments: &str) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        let args = serde_json::from_str::<SayArgs>(arguments)?;
        let message = SayMessage {
            respond_to: tx,
            content: args.content,
            voice: args.voice,
        };

        self.sender
            .send(message)
            .await
            .context("Sending command to say actor")?;
        rx.await.context("getting results from say actor")
    }

    pub fn name(&self) -> &'static str {
        "say"
    }

    pub fn definition(&self) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name(),
                "description": "This tool gives you a voice, allowing you to speak out loud. See the description of the voice parameter for what voice to choose.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "The text that will be spoken out loud. This should be formatted to be pronounced properly without any emoji."
                        },
                        "voice": {
                            "type": "string",
                            "enum": [
                                "Albert",
                                "Alex",
                                "Alice",
                                "Alice (Enhanced)",
                                "Soumya",
                                "Alva",
                                "Aman",
                                "Amélie",
                                "Amélie (Premium)",
                                "Amira",
                                "Angélica (Enhanced)",
                                "Anna",
                                "Aru",
                                "Aude",
                                "Aude (Enhanced)",
                                "Audrey (Premium)",
                                "Aurélie (Enhanced)",
                                "Ava (Premium)",
                                "Bad News",
                                "Bahh",
                                "Bells",
                                "Boing",
                                "Bubbles",
                                "Carmit",
                                "Cellos",
                                "Chantal (Enhanced)",
                                "Damayanti",
                                "Daniel",
                                "Daria",
                                "Wobble",
                                "Eddy (German (Germany))",
                                "Eddy (English (UK))",
                                "Eddy (English (US))",
                                "Eddy (Spanish (Spain))",
                                "Eddy (Spanish (Mexico))",
                                "Eddy (Finnish (Finland))",
                                "Eddy (French (Canada))",
                                "Eddy (French (France))",
                                "Eddy (Italian (Italy))",
                                "Eddy (Japanese (Japan))",
                                "Eddy (Korean (South Korea))",
                                "Eddy (Portuguese (Brazil))",
                                "Eddy (Chinese (China mainland))",
                                "Eddy (Chinese (Taiwan))",
                                "Ellen",
                                "Emma (Enhanced)",
                                "Emma (Premium)",
                                "Federica (Premium)",
                                "Flo (German (Germany))",
                                "Flo (English (UK))",
                                "Flo (English (US))",
                                "Flo (Spanish (Spain))",
                                "Flo (Spanish (Mexico))",
                                "Flo (Finnish (Finland))",
                                "Flo (French (Canada))",
                                "Flo (French (France))",
                                "Flo (Italian (Italy))",
                                "Flo (Japanese (Japan))",
                                "Flo (Korean (South Korea))",
                                "Flo (Portuguese (Brazil))",
                                "Flo (Chinese (China mainland))",
                                "Flo (Chinese (Taiwan))",
                                "Francisca (Enhanced)",
                                "Fred",
                                "Geeta",
                                "Good News",
                                "Grandma (German (Germany))",
                                "Grandma (English (UK))",
                                "Grandma (English (US))",
                                "Grandma (Spanish (Spain))",
                                "Grandma (Spanish (Mexico))",
                                "Grandma (Finnish (Finland))",
                                "Grandma (French (Canada))",
                                "Grandma (French (France))",
                                "Grandma (Italian (Italy))",
                                "Grandma (Japanese (Japan))",
                                "Grandma (Korean (South Korea))",
                                "Grandma (Portuguese (Brazil))",
                                "Grandma (Chinese (China mainland))",
                                "Grandma (Chinese (Taiwan))",
                                "Grandpa (German (Germany))",
                                "Grandpa (English (UK))",
                                "Grandpa (English (US))",
                                "Grandpa (Spanish (Spain))",
                                "Grandpa (Spanish (Mexico))",
                                "Grandpa (Finnish (Finland))",
                                "Grandpa (French (Canada))",
                                "Grandpa (French (France))",
                                "Grandpa (Italian (Italy))",
                                "Grandpa (Japanese (Japan))",
                                "Grandpa (Korean (South Korea))",
                                "Grandpa (Portuguese (Brazil))",
                                "Grandpa (Chinese (China mainland))",
                                "Grandpa (Chinese (Taiwan))",
                                "Jester",
                                "Ioana",
                                "Isabela (Enhanced)",
                                "Jacques",
                                "Joana",
                                "Junior",
                                "Kanya",
                                "Karen",
                                "Kathy",
                                "Kyoko",
                                "Lana",
                                "Laura",
                                "Lekha",
                                "Lesya",
                                "Linh",
                                "Luciana",
                                "Majed",
                                "Tünde",
                                "Marisol (Enhanced)",
                                "Meijia",
                                "Melina",
                                "Melina (Enhanced)",
                                "Milena",
                                "Moira",
                                "Mónica",
                                "Mónica (Enhanced)",
                                "Montse",
                                "Nora",
                                "Ona",
                                "Organ",
                                "Paulina",
                                "Paulina (Enhanced)",
                                "Piya",
                                "Superstar",
                                "Ralph",
                                "Reed (German (Germany))",
                                "Reed (English (UK))",
                                "Reed (English (US))",
                                "Reed (Spanish (Spain))",
                                "Reed (Spanish (Mexico))",
                                "Reed (Finnish (Finland))",
                                "Reed (French (Canada))",
                                "Reed (Italian (Italy))",
                                "Reed (Japanese (Japan))",
                                "Reed (Korean (South Korea))",
                                "Reed (Portuguese (Brazil))",
                                "Reed (Chinese (China mainland))",
                                "Reed (Chinese (Taiwan))",
                                "Rishi",
                                "Rocko (German (Germany))",
                                "Rocko (English (UK))",
                                "Rocko (English (US))",
                                "Rocko (Spanish (Spain))",
                                "Rocko (Spanish (Mexico))",
                                "Rocko (Finnish (Finland))",
                                "Rocko (French (Canada))",
                                "Rocko (French (France))",
                                "Rocko (Italian (Italy))",
                                "Rocko (Japanese (Japan))",
                                "Rocko (Korean (South Korea))",
                                "Rocko (Portuguese (Brazil))",
                                "Rocko (Chinese (China mainland))",
                                "Rocko (Chinese (Taiwan))",
                                "Samantha",
                                "Samantha (Enhanced)",
                                "Sandy (German (Germany))",
                                "Sandy (English (UK))",
                                "Sandy (English (US))",
                                "Sandy (Spanish (Spain))",
                                "Sandy (Spanish (Mexico))",
                                "Sandy (Finnish (Finland))",
                                "Sandy (French (Canada))",
                                "Sandy (French (France))",
                                "Sandy (Italian (Italy))",
                                "Sandy (Japanese (Japan))",
                                "Sandy (Korean (South Korea))",
                                "Sandy (Portuguese (Brazil))",
                                "Sandy (Chinese (China mainland))",
                                "Sandy (Chinese (Taiwan))",
                                "Sara",
                                "Satu",
                                "Shelley (German (Germany))",
                                "Shelley (English (UK))",
                                "Shelley (English (US))",
                                "Shelley (Spanish (Spain))",
                                "Shelley (Spanish (Mexico))",
                                "Shelley (Finnish (Finland))",
                                "Shelley (French (Canada))",
                                "Shelley (French (France))",
                                "Shelley (Italian (Italy))",
                                "Shelley (Japanese (Japan))",
                                "Shelley (Korean (South Korea))",
                                "Shelley (Portuguese (Brazil))",
                                "Shelley (Chinese (China mainland))",
                                "Shelley (Chinese (Taiwan))",
                                "Sinji",
                                "Soledad (Enhanced)",
                                "Tara",
                                "Tessa",
                                "Thomas",
                                "Tina",
                                "Tingting",
                                "Trinoids",
                                "Vani",
                                "Whisper",
                                "Xander",
                                "Jimena (Enhanced)",
                                "Yelda",
                                "Yuna",
                                "Zarvox",
                                "Zosia",
                        "Zuzana"
                            ],
                            "description": "The voice that you want to use when speaking out loud. This must be one of the provided voices in the tool voice parameter."
                        }
                    }
                }
            }
        })
    }

    pub fn system_prompt(&self) -> String {
        let name = self.name();

        format!(
            "\n{name} is a tool you have access to. It allows you to Speak out loud using one of the provided voices. If you use this tool, then you must use pronouncable content without any emoji.\n"
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct SayArgs {
    pub content: String,
    pub voice: String,
}
