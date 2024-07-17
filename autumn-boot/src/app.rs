use crate::{config::env, error::Result, log};
use anyhow::Context;
use tokio::task::JoinHandle;
use toml::Table;
use tracing::debug;

use crate::{config, plugin::Plugin};
use std::{
    any::{self, Any},
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

pub type Registry<T> = HashMap<String, Box<T>>;

pub struct App {
    /// Plugin
    pub(crate) plugin_registry: Registry<dyn Plugin>,
    /// Component
    components: Registry<dyn Any>,
    /// Path of config file
    pub(crate) config_path: PathBuf,
    /// Configuration read from `config_path`
    config: Table,
    /// task
    schedulers: Vec<JoinHandle<Result<String>>>,
}

unsafe impl Send for App {}
unsafe impl Sync for App {}

impl App {
    pub fn new() -> Self {
        log::init_log();
        App::default()
    }

    /// add plugin
    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        debug!("added plugin: {}", plugin.name());
        let plugin_name = plugin.name().to_string();
        if self.plugin_registry.contains_key(plugin.name()) {
            panic!("Error adding plugin {plugin_name}: plugin was already added in application")
        }
        self.plugin_registry.insert(plugin_name, Box::new(plugin));
        self
    }

    /// Returns `true` if the [`Plugin`] has already been added.
    pub fn is_plugin_added<T>(&self) -> bool
    where
        T: Plugin,
    {
        self.plugin_registry.contains_key(any::type_name::<T>())
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
    pub fn get_config<T>(&self, plugin: &impl Plugin) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let prefix = plugin.config_prefix();
        let table = match self.config.get(prefix) {
            Some(toml::Value::Table(table)) => table.to_owned(),
            _ => Table::new(),
        };
        return Ok(T::deserialize(table.to_owned()).with_context(|| {
            format!(
                "Failed to deserialize the configuration of plugin {}",
                plugin.name()
            )
        })?);
    }

    ///
    pub fn add_component<T>(&mut self, component: T) -> &mut Self
    where
        T: Sized + Any,
    {
        let component_name = std::any::type_name::<T>();
        debug!("added component: {}", component_name);
        if self.components.contains_key(component_name) {
            panic!("Error adding component {component_name}: component was already added in application")
        }
        let component_name = component_name.to_string();
        self.components.insert(component_name, Box::new(component));
        self
    }

    ///
    pub fn get_component<T>(&self) -> Option<&T>
    where
        T: Sized + Any,
    {
        let component_name = std::any::type_name::<T>();
        self.components.get(component_name)?.downcast_ref()
    }

    pub fn add_scheduler(&mut self, scheduler: JoinHandle<Result<String>>) -> &mut Self {
        self.schedulers.push(scheduler);
        self
    }

    /// Running
    pub async fn run(&mut self) {
        if let Err(e) = self.inner_run().await {
            tracing::error!("{:?}", e);
        }
    }

    async fn inner_run(&mut self) -> Result<()> {
        // 1. read env variable
        let env = env::init()?;

        // 2. load yaml config
        self.config = config::load_config(self, env)?;

        // 3. build plugin
        self.build_plugins().await;

        // 4. schedule
        self.schedule().await
    }

    async fn build_plugins(&mut self) {
        let registry = std::mem::take(&mut self.plugin_registry);
        let mut to_register = registry.values().collect::<Vec<_>>();
        let mut registered: HashSet<String> = HashSet::new();

        while !to_register.is_empty() {
            let mut progress = false;
            let mut next_round = vec![];

            for plugin in to_register {
                let deps = plugin.dependencies();
                if deps.iter().all(|dep| registered.contains(*dep)) {
                    plugin.build(self).await;
                    registered.insert(plugin.name().to_string());
                    tracing::info!("{} plugin registered", plugin.name());
                    progress = true;
                } else {
                    next_round.push(plugin);
                }
            }

            if !progress {
                panic!("Cyclic dependency detected or missing dependencies for some plugins");
            }

            to_register = next_round;
        }
        self.plugin_registry = registry;
    }

    async fn schedule(&mut self) -> Result<()> {
        while let Some(task) = self.schedulers.pop() {
            match task.await? {
                Err(e) => tracing::error!("{}", e),
                Ok(msg) => tracing::info!("scheduled result: {}", msg),
            }
        }
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            plugin_registry: Default::default(),
            config_path: Path::new("./config/app.toml").to_path_buf(),
            config: Default::default(),
            components: Default::default(),
            schedulers: Default::default(),
        }
    }
}
