use anathema::{component::Component, state::State};

use crate::app::AppMessage;

pub struct Router;

#[derive(Debug, State, Default)]
pub struct RouterState {}

impl Component for Router {
    type State = RouterState;

    type Message = AppMessage;

    fn accept_focus(&self) -> bool {
        false
    }
}
