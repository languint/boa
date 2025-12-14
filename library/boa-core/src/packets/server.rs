pub mod open_result;
pub mod output;
pub mod runner_event;
pub mod server_error;

use serde::{Deserialize, Serialize};

use crate::packets::server::{
    open_result::OpenResultPacket, output::OutputPacket, runner_event::RunnerEventPacket,
    server_error::ServerErrorPacket,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerPacket {
    OpenResult(OpenResultPacket),

    Output(OutputPacket),
    Event(RunnerEventPacket),
    Error(ServerErrorPacket),
}
