use crate::error::{AppError, Result};
use anyhow::Context;
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
            Ok(var) => Self::from_string(var),
            Err(_) => Self::Dev,
        }
    }

    pub fn from_string(str: String) -> Self {
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
        Ok(path) => log::debug!(
            "Loaded the environment variable file under the path: \"{:?}\"",
            path
        ),
        Err(e) => log::debug!("Environment variable file not found: {}", e),
    }

    Ok(Env::from_env())
}
