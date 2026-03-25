mod user_input;
mod chat_history;

use anathema::runtime::Builder;
use eyre::Result;

use crate::components::{chat_history::{ChatHistory, ChatHistoryState}, user_input::{UserInput, UserInputState}};

pub fn register(builder: &mut Builder<()>) -> Result<()> {
    builder.component("user_input", "templates/components/user_input.aml", UserInput, UserInputState::default())?;
    builder.component("chat_history", "templates/components/chat_history.aml", ChatHistory, ChatHistoryState::default())?;

    Ok(())
}
