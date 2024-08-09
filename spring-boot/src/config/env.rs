use anyhow::Context;

use crate::error::{AppError, Result};
use std::{
    env,
    ffi::OsStr,
    io::ErrorKind,
    path::{Path, PathBuf},
};

/// App environment
#[derive(Debug)]
pub enum Env {
    /// Development
    Dev,
    /// Test
    Test,
    /// production
    Prod,
}

impl Env {
    pub fn from_env() -> Self {
        match env::var("SPRING_ENV") {
            Ok(var) => Self::from_str(var),
            Err(_) => Self::Dev,
        }
    }

    pub fn from_str(str: String) -> Self {
        match str {
            str if str.eq_ignore_ascii_case("dev") => Self::Dev,
            str if str.eq_ignore_ascii_case("test") => Self::Test,
            str if str.eq_ignore_ascii_case("prod") => Self::Prod,
            _ => Self::Dev,
        }
    }

    pub(crate) fn get_config_path(&self, path: &Path) -> Result<PathBuf> {
        let stem = path.file_stem().and_then(OsStr::to_str).unwrap_or("");
        let ext = path.extension().and_then(OsStr::to_str).unwrap_or("");
        let canonicalize = path
            .canonicalize()
            .with_context(|| format!("canonicalize {:?} failed", path))?;
        let parent = canonicalize
            .parent()
            .ok_or_else(|| AppError::from_io(ErrorKind::NotFound, "config file path not found"))?;
        Ok(match self {
            Self::Dev => parent.join(format!("{}-dev.{}", stem, ext)),
            Self::Test => parent.join(format!("{}-test.{}", stem, ext)),
            Self::Prod => parent.join(format!("{}-prod.{}", stem, ext)),
        })
    }
}

pub fn init() -> Result<Env> {
    match dotenvy::dotenv() {
        Ok(path) => tracing::debug!(
            "Loaded the environment variable file under the path: \"{}\"",
            path.to_str().unwrap()
        ),
        Err(e) => tracing::debug!("Environment variable file not found: {}", e),
    }

    Ok(Env::from_env())
}
