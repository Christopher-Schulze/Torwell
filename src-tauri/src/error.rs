use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize, Error)]
pub enum Error {
    #[error("Tor Error: {0}")]
    Tor(String),

    #[error("I/O Error: {0}")]
    Io(String),

    #[error("Tauri Error: {0}")]
    Tauri(String),

    #[error("Client not initialized")]
    NotConnected,

    #[error("Client is already connected")]
    AlreadyConnected,

    #[error("Connection timed out")]
    Timeout,

    #[error("No circuit available")]
    NoCircuit,

    #[error("Failed to bootstrap Tor: {0}")]
    Bootstrap(String),

    #[error("Failed to obtain network directory: {0}")]
    NetDir(String),

    #[error("Circuit operation failed: {0}")]
    Circuit(String),

    #[error("Identity change failed: {0}")]
    Identity(String),
}

impl From<arti_client::Error> for Error {
    fn from(err: arti_client::Error) -> Self {
        Error::Tor(err.to_string())
    }
}

impl From<tauri::Error> for Error {
    fn from(err: tauri::Error) -> Self {
        Error::Tauri(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<tor_netdir::Error> for Error {
    fn from(err: tor_netdir::Error) -> Self {
        Error::Tor(err.to_string())
    }
}

impl From<tor_circmgr::Error> for Error {
    fn from(err: tor_circmgr::Error) -> Self {
        Error::Tor(err.to_string())
    }
}

impl From<tor_proto::Error> for Error {
    fn from(err: tor_proto::Error) -> Self {
        Error::Tor(err.to_string())
    }
}
