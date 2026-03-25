use anathema::{component::Component, state::{List, State, Value}};

use crate::app::AppMessage;

pub struct ChatHistory;

impl Component for ChatHistory {
    type State = ChatHistoryState;

    type Message = AppMessage;

    fn on_message(
            &mut self,
            message: Self::Message,
            state: &mut Self::State,
            mut children: anathema::component::Children<'_, '_>,
            mut context: anathema::component::Context<'_, '_, Self::State>,
        ) {
        match message {
            AppMessage::UserPrompt(prompt) => {
                state.senders.push("User".to_owned());
                state.contents.push(prompt);
            }
        }
    }

    fn accept_focus(&self) -> bool {
        false
    }
}

#[derive(State, Debug, Default)]
pub struct ChatHistoryState {
    senders: Value<List<String>>,
    contents: Value<List<String>>,
}
