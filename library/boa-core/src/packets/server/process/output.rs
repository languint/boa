use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessOutputPacket {
    StdOut(String),
    StdErr(String),
}
