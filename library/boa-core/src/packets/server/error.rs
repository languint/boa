use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerError {
    InvalidContainerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerErrorPacket {
    pub err: ServerError,
    pub message: String,
}
