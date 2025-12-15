use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerError {
    InvalidJson,
    InvalidContainerId,
    ProcessStartFailed,
    TempFileCreationFailed,
    UploadAlreadyInProgress,
    UploadFailed,
    UnexpectedBinaryFrame,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerErrorPacket {
    pub err: ServerError,
    pub message: String,
}
