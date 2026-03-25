#[derive(Debug, Clone)]
pub enum BBAiCommand {
    Prompt(String),
    ResetContext,
}
