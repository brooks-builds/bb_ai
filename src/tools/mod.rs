pub mod git_diff;
pub mod git_status;
pub mod random_number;
pub mod read_file;
pub mod spawn_agent;
pub mod list_files;

use tokio::sync::oneshot;

#[derive(Debug)]
pub struct ToolMessage {
    pub name: String,
    pub arguments: String,
    pub send_to: oneshot::Sender<String>,
}
