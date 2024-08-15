pub mod env;

use crate::app::AppBuilder;
use crate::error::{AppError, Result};
use anyhow::Context;
use env::Env;
use serde_toml_merge::merge_tables;
pub use spring_macros::Configurable;
use std::fs;
use toml::Table;

pub trait Configurable {
    /// Prefix used to read toml configuration.
    /// If you need to load external configuration, you need to rewrite this method
    fn config_prefix(&self) -> &str;
}

/// load toml config
pub(crate) fn load_config(app: &AppBuilder, env: Env) -> Result<Table> {
    let main_path = app.config_path.as_path();
    let config_file = fs::read_to_string(main_path);
    if let Err(e) = config_file {
        tracing::warn!("Failed to read configuration file {:?}: {}", main_path, e);
        return Ok(Table::new());
    }

    let main_toml_str = config_file.unwrap();
    let main_table = toml::from_str::<Table>(main_toml_str.as_str())
        .with_context(|| format!("Failed to parse the toml file at path {:?}", main_path))?;

    let config_table = match env.get_config_path(main_path) {
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
                .map_err(|e| AppError::TomlMergeError(format!("merge toml error: {}", e)))
                .with_context(|| {
                    format!("Failed to merge files {:?} and {:?}", main_path, env_path)
                })?
        }
        Err(_) => {
            tracing::debug!("{:?} config not found", env);
            main_table
        }
    };

    Ok(config_table)
}
