use async_openai::types::chat::ChatCompletionTools;

pub mod random_number;

pub trait Tool :Send+Sync {
    type SendToResult: Send+Sync;

    fn definition(&self) -> ChatCompletionTools;
    fn name(&self) -> String;
    async fn send_to(&self, min: i32, max: i32) -> eyre::Result<Self::SendToResult>;
}
