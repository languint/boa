pub mod control_signal;

use serde::{Deserialize, Serialize};

use crate::packets::client::control_signal::ControlSignalPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientPacket {
    ControlSignal(ControlSignalPacket),

    Close,
}
