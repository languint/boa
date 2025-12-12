use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlSignal {
    /// Send a SIGINT to the container
    Interrupt,
    /// Send a SIGTERM to the container
    Terminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlSignalPacket {
    pub control_signal: ControlSignal,
}
