use super::env::Env;
use super::{ConfigRegistry, Configurable};
use crate::error::{AppError, Result};
use anyhow::Context;
use serde::de::DeserializeOwned;
use serde_toml_merge::merge_tables;
use std::fs;
use std::path::Path;
use toml::Table;

#[derive(Default)]
pub struct TomlConfigRegistry {
    config: Table,
}

impl ConfigRegistry for TomlConfigRegistry {
    fn get_config<T>(&self) -> Result<T>
    where
        T: DeserializeOwned + Configurable,
    {
        let prefix = T::config_prefix();
        let table = self.get_by_prefix(prefix);
        T::deserialize(table.to_owned()).map_err(|e| AppError::DeserializeErr(prefix, e))
    }
}

#[cfg(feature = "inline_file")]
fn get_inline_str(inline_str: &str) -> &str {
    inline_str
}

#[cfg(not(feature = "inline_file"))]
fn get_profile(config_path: &Path) -> Result<String> {
    let config_file_content = std::fs::read_to_string(config_path);
    let main_toml_str = match config_file_content {
        Err(e) => {
            log::warn!("Failed to read configuration file {:?}: {}", config_path, e);
            return Ok(Table::new().to_string());
        }
        Ok(content) => content,
    };
    Ok(main_toml_str)
}

impl TomlConfigRegistry {
    pub fn new(config_path: &Path, env: Env, inline_str: &str) -> Result<Self> {
        let config = Self::load_config(config_path, env, inline_str)?;
        Ok(Self { config })
    }

    pub fn get_by_prefix(&self, prefix: &str) -> Table {
        match self.config.get(prefix) {
            Some(toml::Value::Table(table)) => table.clone(),
            _ => Table::new(),
        }
    }

    /// load toml config
    fn load_config(config_path: &Path, env: Env, inline_str: &str) -> Result<Table> {
        // let config_file_content = fs::read_to_string(config_path);
        // let main_toml_str = match config_file_content {
        //     Err(e) => {
        //         log::warn!("Failed to read configuration file {:?}: {}", config_path, e);
        //         return Ok(Table::new());
        //     }
        //     Ok(content) => content,
        // };

        #[cfg(feature = "inline_file")]
        let main_toml_str = get_inline_str(inline_str);

        #[cfg(not(feature = "inline_file"))]
        let main_toml_str = get_profile(config_path)?;

        let main_table = toml::from_str::<Table>(&main_toml_str)
            .with_context(|| format!("Failed to parse the toml file at path {:?}", config_path))?;

        let config_table: Table = match env.get_config_path(config_path) {
            Ok(env_path) => {
                let env_path = env_path.as_path();
                if !env_path.exists() {
                    return Ok(main_table);
                }
                log::info!("The profile of the {:?} environment is active", env);

                let env_toml_str = fs::read_to_string(env_path)
                    .with_context(|| format!("Failed to read configuration file {:?}", env_path))?;
                let env_table =
                    toml::from_str::<Table>(env_toml_str.as_str()).with_context(|| {
                        format!("Failed to parse the toml file at path {:?}", env_path)
                    })?;
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
}

#[allow(unused_imports)]
mod tests {
    use super::Env;
    use super::TomlConfigRegistry;
    use crate::error::Result;
    use std::fs;

    #[test]
    fn test_load_config() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;

        let foo = temp_dir.path().join("foo.toml");
        #[rustfmt::skip]
        let _ = fs::write(&foo,r#"
        [group]
        key = "A"
        "#,
        );

        let table = TomlConfigRegistry::new(&foo, Env::from_string("dev"), "")?;
        let group = table.get_by_prefix("group");
        assert_eq!(group.get("key").unwrap().as_str(), Some("A"));

        // test merge
        let foo_dev = temp_dir.path().join("foo-dev.toml");
        #[rustfmt::skip]
        let _ = fs::write(foo_dev,r#"
        [group]
        key = "OOOOA"
        "#,
        );

        let table = TomlConfigRegistry::new(&foo, Env::from_string("dev"), "")?;
        let group = table.get_by_prefix("group");
        assert_eq!(group.get("key").unwrap().as_str(), Some("OOOOA"));

        Ok(())
    }
}
