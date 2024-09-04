pub mod env;

use crate::error::{AppError, Result};
use anyhow::Context;
use env::Env;
use serde_toml_merge::merge_tables;
pub use spring_macros::Configurable;
use std::fs;
use std::path::Path;
use toml::Table;

pub trait Configurable {
    /// Prefix used to read toml configuration.
    /// If you need to load external configuration, you need to rewrite this method
    fn config_prefix(&self) -> &str;
}

/// load toml config
pub(crate) fn load_config(config_path: &Path, env: Env) -> Result<Table> {
    let config_file_content = fs::read_to_string(config_path);
    let main_toml_str = match config_file_content {
        Err(e) => {
            log::warn!("Failed to read configuration file {:?}: {}", config_path, e);
            return Ok(Table::new());
        }
        Ok(content) => content,
    };

    let main_table = toml::from_str::<Table>(main_toml_str.as_str())
        .with_context(|| format!("Failed to parse the toml file at path {:?}", config_path))?;

    let config_table: toml::map::Map<String, toml::Value> = match env.get_config_path(config_path) {
        Ok(env_path) => {
            let env_path = env_path.as_path();
            if !env_path.exists() {
                return Ok(main_table);
            }

            let env_toml_str = fs::read_to_string(env_path)
                .with_context(|| format!("Failed to read configuration file {:?}", env_path))?;
            let env_table = toml::from_str::<Table>(env_toml_str.as_str())
                .with_context(|| format!("Failed to parse the toml file at path {:?}", env_path))?;
            merge_tables(main_table, env_table)
                .map_err(|e| AppError::TomlMergeError(e.to_string()))
                .with_context(|| {
                    format!("Failed to merge files {:?} and {:?}", config_path, env_path)
                })?
        }
        Err(_) => {
            log::debug!("{:?} config not found", env);
            main_table
        }
    };

    Ok(config_table)
}
