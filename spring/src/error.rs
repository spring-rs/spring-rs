use std::io::{self, ErrorKind};
use thiserror::Error;

/// Spring custom error type
#[derive(Error, Debug)]
pub enum AppError {
    /// `.env` file reading failed
    #[error(transparent)]
    EnvError(#[from] dotenvy::Error),

    /// File IO Error
    #[error(transparent)]
    IOError(#[from] io::Error),

    /// toml file parsing error
    #[error(transparent)]
    TomlParseError(#[from] toml::de::Error),

    /// Configuration merge error in toml file
    #[error("merge toml error: {0}")]
    TomlMergeError(String),

    /// tokio asynchronous task join failed
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),

    /// Deserialization of configuration in toml file to rust struct failed
    #[error("Failed to deserialize the configuration of prefix \"{0}\": {1}")]
    DeserializeErr(&'static str, toml::de::Error),

    /// Other runtime errors
    #[error(transparent)]
    OtherError(#[from] anyhow::Error),
}

impl AppError {
    /// Failed to read file io
    pub fn from_io(kind: ErrorKind, msg: &str) -> Self {
        AppError::IOError(io::Error::new(kind, msg))
    }
}

/// Contains the return value of AppError
pub type Result<T> = std::result::Result<T, AppError>;
