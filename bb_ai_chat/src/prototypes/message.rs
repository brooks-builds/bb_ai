use anathema::component::Component;

pub struct Message;

impl Component for Message {
    type State = ();

    type Message = ();

    fn accept_focus(&self) -> bool {
        false
    }
}
