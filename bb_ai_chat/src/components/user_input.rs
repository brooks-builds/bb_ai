use anathema::{component::Component, state::State};
use bb_anathema_components::input::BBInputMessage;

use crate::app::AppMessage;

pub struct UserInput;

#[derive(Debug, State, Default)]
pub struct UserInputState {}

impl Component for UserInput {
    type State = UserInputState;

    type Message = AppMessage;

    fn on_event(
            &mut self,
            event: &mut anathema::component::UserEvent<'_>,
            state: &mut Self::State,
            mut children: anathema::component::Children<'_, '_>,
            mut context: anathema::component::Context<'_, '_, Self::State>,
        ) {
        if event.name() == "user_input" {
            event.stop_propagation();

            let user_input = event.data_checked::<String>().cloned().unwrap();
            let message = AppMessage::UserPrompt(user_input);

            context.components.by_name("index").send(message);
            context.components.by_name("BBInput").send(BBInputMessage::Clear);
        }
    }

    fn accept_focus(&self) -> bool {
        false
    }
}

