pub mod error;
pub mod process;

use serde::{Deserialize, Serialize};

use crate::packets::server::{
    error::ServerErrorPacket,
    process::{
        ProcessCloseResultPacket, ProcessEventPacket, ProcessOpenResultPacket, ProcessOutputPacket,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerPacket {
    ProcessOpenResult(ProcessOpenResultPacket),
    ProcessCloseResult(ProcessCloseResultPacket),
    ProcessOutput(ProcessOutputPacket),
    ProcessEvent(ProcessEventPacket),

    ServerError(ServerErrorPacket),
}
