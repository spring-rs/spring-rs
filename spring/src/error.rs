use std::io::{self, ErrorKind};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    EnvError(#[from] dotenvy::Error),

    #[error(transparent)]
    IOError(#[from] io::Error),

    #[error(transparent)]
    TomlParseError(#[from] toml::de::Error),

    #[error("merge toml error: {0}")]
    TomlMergeError(String),

    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Failed to deserialize the configuration of prefix \"{0}\": {1}")]
    DeserializeErr(&'static str, toml::de::Error),

    #[error("config of prefix \"{0}\" not found")]
    ConfigNotFoundErr(&'static str),

    #[error(transparent)]
    OtherError(#[from] anyhow::Error),
}

impl AppError {
    pub fn from_io(kind: ErrorKind, msg: &str) -> Self {
        AppError::IOError(io::Error::new(kind, msg))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
