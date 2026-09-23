use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "code", content = "message")]
pub enum ByteError {
    #[error("Local storage error: {0}")]
    Io(String),
    #[error("Stored configuration could not be decoded: {0}")]
    Config(String),
    #[error("Window operation failed: {0}")]
    Window(String),
    #[error("Telemetry source failed: {0}")]
    Telemetry(String),
}

impl From<std::io::Error> for ByteError {
    fn from(value: std::io::Error) -> Self { Self::Io(value.to_string()) }
}

impl From<serde_json::Error> for ByteError {
    fn from(value: serde_json::Error) -> Self { Self::Config(value.to_string()) }
}
