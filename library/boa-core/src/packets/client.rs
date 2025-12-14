pub mod close;
pub mod control_signal;
pub mod open;

use serde::{Deserialize, Serialize};

use crate::packets::client::{
    close::ClosePacket, control_signal::ControlSignalPacket, open::OpenPacket,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientPacket {
    ControlSignal(ControlSignalPacket),

    Open(OpenPacket),
    Close(ClosePacket),
}
