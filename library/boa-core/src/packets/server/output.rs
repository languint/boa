use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputPacket {
    StdOut(String),
    StdErr(String),
}
