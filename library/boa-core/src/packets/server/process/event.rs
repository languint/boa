use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessEventPacket {
    Started,
    Finished { exit_code: i64 },
    TimedOut,
}
