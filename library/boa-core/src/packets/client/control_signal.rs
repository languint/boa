use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlSignal {
    /// Send a SIGINT to the container
    Interrupt,
    /// Send a SIGTERM to the container
    Terminate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlSignalPacket {
    pub container_id: String,
    pub control_signal: ControlSignal,
}
