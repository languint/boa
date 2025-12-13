use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerErrorPacket {
    pub message: String,
    pub fatal: bool,
}
