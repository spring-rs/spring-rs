use crate::{
    config::env,
    error::{AppError, Result},
};
use anyhow::Context;
use toml::Table;
use tracing::debug;

use crate::{config, plugin::Plugin};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub struct App {
    /// List of plugins that have been added.
    pub(crate) plugin_registry: Vec<Box<dyn Plugin>>,
    /// The names of plugins that have been added to this app. (used to track duplicates and
    /// already-registered plugins)
    pub(crate) plugin_names: HashSet<String>,
    /// Path of config file
    pub(crate) config_path: PathBuf,
    /// Configuration read from `config_path`
    pub(crate) config: Table,
}

impl App {
    pub fn new() -> Self {
        App::default()
    }

    /// add plugin
    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        debug!("added plugin: {}", plugin.name());
        if plugin.is_unique() && self.plugin_names.contains(plugin.name()) {
            let plugin_name = plugin.name().to_string();
            panic!("Error adding plugin {plugin_name}: : plugin was already added in application")
        }
        self.plugin_names.insert(plugin.name().to_string());
        self.plugin_registry.push(Box::new(plugin));
        self
    }

    /// Returns `true` if the [`Plugin`] has already been added.
    pub fn is_plugin_added<T>(&self) -> bool
    where
        T: Plugin,
    {
        self.plugin_names.contains(std::any::type_name::<T>())
    }

    /// The path of the configuration file, default is `./config/app.toml`.
    /// The application automatically reads the environment configuration file
    /// in the same directory according to the `AUTUMN_ENV` environment variable,
    /// such as `./config/app-dev.toml`.
    /// The environment configuration file has a higher priority and will
    /// overwrite the configuration items of the main configuration file.
    ///
    /// For specific supported environments, see the [Env](./config/env) enum.
    pub fn config_file(&mut self, config_path: &str) -> &mut Self {
        self.config_path = Path::new(config_path).to_path_buf();
        self
    }

    /// Get the configuration items of the plugin according to the plugin's `config_prefix`
    pub fn get_config<T>(&mut self, plugin: impl Plugin) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let prefix = plugin.config_prefix();
        if let Some(toml::Value::Table(table)) = self.config.get(prefix) {
            return Ok(T::deserialize(table.to_owned()).with_context(|| {
                format!(
                    "Failed to deserialize the configuration of plugin {}",
                    plugin.name()
                )
            })?);
        }
        let plugin_name = plugin.name();
        Err(AppError::ConfigError(format!(
            "The {} prefix configuration for the {} plugin was not found",
            prefix, plugin_name
        )))
    }

    /// Running
    pub fn run(&mut self) -> Result<()> {
        // 1. read env variable
        let env = env::init()?;

        // 2. load yaml config
        self.config = config::load_config(self, env)?;

        // 3. build plugin
        self.build_plugins();

        // 4. finish
        self.finish();

        // 5. cleanup
        self.cleanup();

        Ok(())
    }

    fn build_plugins(&mut self) {
        let plugins = std::mem::take(&mut self.plugin_registry);
        for plugin in &plugins {
            plugin.build(self);
        }
        self.plugin_registry = plugins;
    }

    fn finish(&mut self) {
        let plugins = std::mem::take(&mut self.plugin_registry);
        for plugin in &plugins {
            plugin.finish(self);
        }
        self.plugin_registry = plugins;
    }

    fn cleanup(&mut self) {
        let plugins = std::mem::take(&mut self.plugin_registry);
        for plugin in &plugins {
            plugin.cleanup(self);
        }
        self.plugin_registry = plugins;
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            plugin_registry: Default::default(),
            plugin_names: Default::default(),
            config_path: Path::new("./config/app.toml").to_path_buf(),
            config: Table::with_capacity(0),
        }
    }
}
