use crate::{config::env, error::Result};
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
    /// path of config file
    pub(crate) config_path: PathBuf,
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
        plugin.build(self);
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

    pub fn finish(&mut self) {
        let plugins = std::mem::take(&mut self.plugin_registry);
        for plugin in &plugins {
            plugin.finish(self);
        }
        self.plugin_registry = plugins;
    }

    pub fn cleanup(&mut self) {
        // plugins installed to main should see all sub-apps
        let plugins = std::mem::take(&mut self.plugin_registry);
        for plugin in &plugins {
            plugin.cleanup(self);
        }
        self.plugin_registry = plugins;
    }

    pub fn run(&mut self) -> Result<()> {
        // 1. read env variable
        let env = env::init()?;

        // 2. load yaml config
        config::load_config(self, env)?;

        // 3. finish
        self.finish();

        // 4. cleanup
        self.cleanup();

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            plugin_registry: Default::default(),
            plugin_names: Default::default(),
            config_path: Path::new("./config/app.toml").to_path_buf(),
        }
    }
}
