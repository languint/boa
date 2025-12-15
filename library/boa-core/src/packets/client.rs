pub mod process;

use serde::{Deserialize, Serialize};

use crate::packets::client::process::{
    ProcessClosePacket, ProcessControlSignalPacket, ProcessOpenPacket,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientPacket {
    ProcessOpen(ProcessOpenPacket),
    ProcessClose(ProcessClosePacket),
    ProcessControlSignal(ProcessControlSignalPacket),

    UploadStart {
        container_id: String,
        path: String,
        size: u64,
    },
    UploadFinish {
        container_id: String,
    },
}
