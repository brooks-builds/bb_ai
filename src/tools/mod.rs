use async_openai::types::chat::ChatCompletionTools;

pub mod random_number;

pub trait Tool :Send+Sync {
    fn definition(&self) -> ChatCompletionTools;
    fn name(&self) -> String;
}
