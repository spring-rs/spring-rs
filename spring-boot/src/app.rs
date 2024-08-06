use crate::{config, plugin::Plugin};
use crate::{
    config::env,
    error::Result,
    log,
    plugin::{component::ComponentRef, PluginRef},
};
use anyhow::Context;
use dashmap::DashMap;
use std::any::Any;
use std::{
    any,
    collections::HashSet,
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
};
use toml::Table;
use tracing::debug;

pub type Registry<T> = DashMap<String, T>;
pub type Scheduler = dyn FnOnce(Arc<App>) -> Box<dyn Future<Output = Result<String>> + Send>;

pub struct App {
    /// Component
    components: Registry<ComponentRef>,
}

pub struct AppBuilder {
    /// Plugin
    pub(crate) plugin_registry: Registry<PluginRef>,
    /// Component
    components: Registry<ComponentRef>,
    /// Path of config file
    pub(crate) config_path: PathBuf,
    /// Configuration read from `config_path`
    config: Table,
    /// task
    schedulers: Vec<Box<Scheduler>>,
}

impl App {
    pub fn new() -> AppBuilder {
        log::init_log();
        AppBuilder::default()
    }

    ///
    pub fn get_component<T>(&self) -> Option<Arc<T>>
    where
        T: Any + Send + Sync,
    {
        let component_name = std::any::type_name::<T>();
        let pair = self.components.get(component_name)?;
        let component_ref = pair.value().clone();
        component_ref.downcast::<T>()
    }

    pub fn get_components(&self) -> Vec<String> {
        self.components.iter().map(|e| e.key().clone()).collect()
    }
}

unsafe impl Send for AppBuilder {}
unsafe impl Sync for AppBuilder {}

impl AppBuilder {
    /// add plugin
    pub fn add_plugin<T: Plugin>(&mut self, plugin: T) -> &mut Self {
        debug!("added plugin: {}", plugin.name());
        let plugin_name = plugin.name().to_string();
        if self.plugin_registry.contains_key(plugin.name()) {
            panic!("Error adding plugin {plugin_name}: plugin was already added in application")
        }
        self.plugin_registry
            .insert(plugin_name, PluginRef::new(plugin));
        self
    }

    /// Returns `true` if the [`Plugin`] has already been added.
    pub fn is_plugin_added<T: Plugin>(&self) -> bool {
        self.plugin_registry.contains_key(any::type_name::<T>())
    }

    /// The path of the configuration file, default is `./config/app.toml`.
    /// The application automatically reads the environment configuration file
    /// in the same directory according to the `spring_ENV` environment variable,
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
        T: any::Any + Send + Sync,
    {
        let component_name = std::any::type_name::<T>();
        debug!("added component: {}", component_name);
        if self.components.contains_key(component_name) {
            panic!("Error adding component {component_name}: component was already added in application")
        }
        let component_name = component_name.to_string();
        self.components
            .insert(component_name, ComponentRef::new(component));
        self
    }

    ///
    pub fn get_component<T>(&self) -> Option<Arc<T>>
    where
        T: Any + Send + Sync,
    {
        let component_name = std::any::type_name::<T>();
        let pair = self.components.get(component_name)?;
        let component_ref = pair.value().clone();
        component_ref.downcast::<T>()
    }

    ///
    pub fn add_scheduler<T>(&mut self, scheduler: T) -> &mut Self
    where
        T: FnOnce(Arc<App>) -> Box<dyn Future<Output = Result<String>> + Send> + 'static,
    {
        self.schedulers.push(Box::new(scheduler));
        self
    }

    /// Running
    pub async fn run(&mut self) {
        match self.inner_run().await {
            Err(e) => {
                tracing::error!("{:?}", e);
            }
            Ok(_app) => {}
        }
    }

    async fn inner_run(&mut self) -> Result<Arc<App>> {
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
        let mut to_register = registry
            .iter()
            .map(|e| e.value().to_owned())
            .collect::<Vec<_>>();
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

    async fn schedule(&mut self) -> Result<Arc<App>> {
        let app = self.build_app();
        while let Some(task) = self.schedulers.pop() {
            let poll_future = task(app.clone());
            let poll_future = Box::into_pin(poll_future);
            match tokio::spawn(poll_future).await? {
                Err(e) => tracing::error!("{}", e),
                Ok(msg) => tracing::info!("scheduled result: {}", msg),
            }
        }
        Ok(app)
    }

    fn build_app(&mut self) -> Arc<App> {
        let components = std::mem::take(&mut self.components);
        Arc::new(App { components })
    }
}

impl Default for AppBuilder {
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
