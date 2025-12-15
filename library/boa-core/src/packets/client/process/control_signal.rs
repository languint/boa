use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessControlSignal {
    /// Send a start request to the container
    Start,
    /// Send a exec request to the container
    Exec(String),
    /// Send a SIGINT to the container
    Interrupt,
    /// Send a SIGTERM to the container
    Terminate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessControlSignalPacket {
    pub container_id: String,
    pub control_signal: ProcessControlSignal,
}
