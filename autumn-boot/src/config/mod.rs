pub mod env;

use std::fs;

use env::Env;
use toml::Table;

use crate::app::App;
use crate::error::{AppError, Result};
use serde_toml_merge::merge_tables;

pub trait ConfigListener {}

/// load toml config
pub(crate) fn load_config(app: &App, env: Env) -> Result<Table> {
    let main_path = app.config_path.as_path();
    let env_path = env.get_config_path(main_path)?;

    let main_toml_str = fs::read_to_string(main_path)?;
    let main_table = toml::from_str::<Table>(main_toml_str.as_str())?;

    let config_table = match fs::read_to_string(env_path) {
        Ok(env_toml_str) => {
            let env_table = toml::from_str::<Table>(env_toml_str.as_str())?;
            merge_tables(main_table, env_table)
                .map_err(|e| AppError::TomlMergeError(format!("merge toml error: {}", e)))?
        }
        Err(_) => {
            tracing::debug!("{:?} config not found", env);
            main_table
        }
    };

    Ok(config_table)
}
