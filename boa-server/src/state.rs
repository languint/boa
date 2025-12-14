use std::sync::Arc;

use bollard::Docker;
use tokio::sync::Mutex;

use crate::container::BoaContainer;

pub type ShareableServerState = Arc<Mutex<ServerState>>;

#[derive(Clone)]
pub struct ServerState {
    pub containers: Vec<BoaContainer>,
    pub docker: Docker,
}

impl ServerState {
    pub fn new(docker: Docker) -> Self {
        Self {
            containers: vec![],
            docker,
        }
    }
}
