pub mod agent;

use std::collections::HashMap;
use crate::agent::AgentHandle;

pub fn create_actors() -> HashMap<String, AgentHandle> {
    let agent_handle = AgentHandle::spawn();
    let mut map = HashMap::new();

    map.insert(agent::NAME.to_owned(), agent_handle);

    map
}
