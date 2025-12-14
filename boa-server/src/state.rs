use std::{collections::HashMap, sync::Arc};

use bollard::Docker;
use tokio::sync::Mutex;

use crate::container::BoaContainer;

pub type ShareableServerState = Arc<Mutex<ServerState>>;

#[derive(Clone)]
pub struct ServerState {
    pub containers: HashMap<String, BoaContainer>,
    pub docker: Docker,

    // Environment vars
    pub server_port: u32,
    pub container_prefix: String,
}

impl ServerState {
    pub fn new(docker: Docker, server_port: u32, container_prefix: String) -> Self {
        Self {
            containers: HashMap::new(),
            docker,

            server_port,
            container_prefix,
        }
    }
}
