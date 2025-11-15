use serde::Serialize;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Serialize, Clone, Error)]
pub enum ConnectionStep {
    #[error("build_config")]
    BuildConfig,
    #[error("bootstrap")]
    Bootstrap,
    #[error("timeout")]
    Timeout,
    #[error("retries_exceeded")]
    RetriesExceeded,
}

#[derive(Debug, Error, Serialize, Clone)]
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

    #[error("Failed to obtain network directory: {source_message}")]
    NetDir { source_message: String },

    #[error("Circuit operation failed: {source_message}")]
    Circuit { source_message: String },

    #[error("Network error: {source_message}")]
    Network { source_message: String },

    #[error("connection failed during {step}: {source_message}")]
    ConnectionFailed {
        step: ConnectionStep,
        source_message: String,
        backtrace: String,
    },

    #[error("identity change failed during {step}: {source_message}")]
    Identity {
        step: String,
        source_message: String,
        backtrace: String,
    },

    #[error("Rate limit exceeded for {0}")]
    RateLimitExceeded(String),

    #[error("configuration error during {step}: {source_message}")]
    ConfigError {
        step: String,
        source_message: String,
        backtrace: String,
    },

    #[error("network failure during {step}: {source_message}")]
    NetworkFailure {
        step: String,
        source_message: String,
        backtrace: String,
    },

    #[error("insecure HTTP access to {host}")]
    InsecureScheme { host: String, url: String },

    #[error("connection failed after {attempts} retries: {error}")]
    RetriesExceeded {
        attempts: u32,
        error: String,
        backtrace: String,
    },

    #[error("bridge parsing failed: {0}")]
    BridgeParse(String),

    #[error("country lookup failed: {0}")]
    Lookup(String),

    #[error("Invalid session token")]
    InvalidToken,

    #[error("GPU error: {0}")]
    Gpu(String),

    #[error("Reqwest error: {source_message}")]
    Reqwest { source_message: String },

    #[error("Arti client error: {source_message}")]
    ArtiClient { source_message: String },
}

impl From<tor_netdir::Error> for Error {
    fn from(err: tor_netdir::Error) -> Self {
        Error::NetDir {
            source_message: err.to_string(),
        }
    }
}

impl From<tor_circmgr::Error> for Error {
    fn from(err: tor_circmgr::Error) -> Self {
        Error::Circuit {
            source_message: err.to_string(),
        }
    }
}

impl From<tor_proto::Error> for Error {
    fn from(err: tor_proto::Error) -> Self {
        Error::Network {
            source_message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest {
            source_message: err.to_string(),
        }
    }
}

impl From<arti_client::Error> for Error {
    fn from(err: arti_client::Error) -> Self {
        Error::ArtiClient {
            source_message: err.to_string(),
        }
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

impl From<String> for Error {
    fn from(err: String) -> Self {
        // This is a catch-all. Consider creating a more specific error variant if possible.
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Io(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
