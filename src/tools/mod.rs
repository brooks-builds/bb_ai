pub mod random_number;

use tokio::sync::oneshot;

#[derive(Debug)]
pub struct ToolMessage {
    pub name: String,
    pub arguments: String,
    pub send_to: oneshot::Sender<String>,
}
