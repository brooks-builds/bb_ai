use anathema::{component::Component, state::{State, Value}};

pub struct App;

#[derive(Debug, State, Default)]
pub struct AppState {
    route: Value<String>,
    chat_state: Value<String>,
}

pub enum AppMessage {
    UserPrompt(String),
}

impl Component for App {
    type State = AppState;

    type Message = AppMessage;

    fn on_mount(
            &mut self,
            state: &mut Self::State,
            mut children: anathema::component::Children<'_, '_>,
            mut context: anathema::component::Context<'_, '_, Self::State>,
        ) {
        state.route.set("home".to_owned());
        state.chat_state.set("waiting_for_user".to_owned());
    }

    fn accept_focus(&self) -> bool {
        false
    }

    fn on_message(
            &mut self,
            message: Self::Message,
            state: &mut Self::State,
            mut children: anathema::component::Children<'_, '_>,
            mut context: anathema::component::Context<'_, '_, Self::State>,
        ) {
        match message {
            AppMessage::UserPrompt(prompt) => {
                let message = AppMessage::UserPrompt(prompt);

                context.components.by_name("chat_history").send(message);
                state.chat_state.set("waiting_for_ai".to_owned());
            },
        }
    }
}
