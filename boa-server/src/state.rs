use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ServerState {
    pub containers: Arc<Mutex<HashMap<String, String>>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            containers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
