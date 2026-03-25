use anathema::{component::Component, state::State};

use crate::app::AppMessage;

pub struct ChatPage;

#[derive(Debug, State, Default)]
pub struct ChatPageState {}

impl Component for ChatPage {
    type State = ChatPageState;

    type Message = AppMessage;

    fn accept_focus(&self) -> bool {
        false
    }
}
