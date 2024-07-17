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

    #[error("{0}")]
    TomlMergeError(String),

    #[error("{0}")]
    ConfigError(String),

    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),

    #[error(transparent)]
    OtherError(#[from] anyhow::Error),
}

impl AppError {
    pub fn from_io(kind: ErrorKind, msg: &str) -> Self {
        AppError::IOError(io::Error::new(kind, msg))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
