use crate::config::env::Env;
use crate::config::toml::TomlConfigRegistry;
use crate::config::ConfigRegistry;
use crate::log::{BoxLayer, LogPlugin};
use crate::plugin::component::ComponentRef;
use crate::plugin::{service, Plugin};
use crate::{
    error::Result,
    plugin::{component::DynComponentRef, PluginRef},
};
use dashmap::DashMap;
use std::any::Any;
use std::{
    any,
    collections::HashSet,
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing_subscriber::Layer;

pub type Registry<T> = DashMap<String, T>;
pub type Scheduler<T> = dyn FnOnce(Arc<App>) -> Box<dyn Future<Output = Result<T>> + Send>;

pub struct App {
    /// Component
    components: Registry<DynComponentRef>,
    config: TomlConfigRegistry,
}

pub struct AppBuilder {
    pub(crate) env: Env,
    /// Path of config file
    pub(crate) config_path: PathBuf,
    /// Tracing Layer
    pub(crate) layers: Vec<BoxLayer>,
    /// Plugin
    pub(crate) plugin_registry: Registry<PluginRef>,
    /// Component
    components: Registry<DynComponentRef>,
    /// Configuration read from `config_path`
    config: TomlConfigRegistry,
    /// task
    schedulers: Vec<Box<Scheduler<String>>>,
    shutdown_hooks: Vec<Box<Scheduler<String>>>,
}

impl App {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> AppBuilder {
        AppBuilder::default()
    }

    /// Get the component of the specified type
    pub fn get_component_ref<T>(&self) -> Option<ComponentRef<T>>
    where
        T: Any + Send + Sync,
    {
        let component_name = std::any::type_name::<T>();
        let pair = self.components.get(component_name)?;
        let component_ref = pair.value().clone();
        component_ref.downcast::<T>()
    }

    pub fn get_component<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let component_ref = self.get_component_ref();
        component_ref.map(|c| T::clone(&c))
    }

    pub fn get_components(&self) -> Vec<String> {
        self.components.iter().map(|e| e.key().clone()).collect()
    }
}

unsafe impl Send for AppBuilder {}
unsafe impl Sync for AppBuilder {}

impl AppBuilder {
    pub fn get_env(&self) -> &Env {
        &self.env
    }

    /// add plugin
    pub fn add_plugin<T: Plugin>(&mut self, plugin: T) -> &mut Self {
        let plugin_name = plugin.name().to_string();
        log::debug!("added plugin: {plugin_name}");
        if plugin.immediately() {
            plugin.immediately_build(self);
            return self;
        }
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
    /// in the same directory according to the `SPRING_ENV` environment variable,
    /// such as `./config/app-dev.toml`.
    /// The environment configuration file has a higher priority and will
    /// overwrite the configuration items of the main configuration file.
    ///
    /// For specific supported environments, see the [Env](../config/env/enum.Env.html) enum.
    pub fn config_file(&mut self, config_path: &str) -> &mut Self {
        self.config_path = Path::new(config_path).to_path_buf();
        self
    }

    /// Add component to the registry
    pub fn add_component<T>(&mut self, component: T) -> &mut Self
    where
        T: Clone + any::Any + Send + Sync,
    {
        let component_name = std::any::type_name::<T>();
        log::debug!("added component: {}", component_name);
        if self.components.contains_key(component_name) {
            panic!("Error adding component {component_name}: component was already added in application")
        }
        let component_name = component_name.to_string();
        self.components
            .insert(component_name, DynComponentRef::new(component));
        self
    }

    /// Get the component of the specified type
    pub fn get_component_ref<T>(&self) -> Option<ComponentRef<T>>
    where
        T: Any + Send + Sync,
    {
        let component_name = std::any::type_name::<T>();
        let pair = self.components.get(component_name)?;
        let component_ref = pair.value().clone();
        component_ref.downcast::<T>()
    }

    /// get cloned component
    pub fn get_component<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let component_ref = self.get_component_ref();
        component_ref.map(|c| T::clone(&c))
    }

    /// add [tracing_subscriber::layer]
    pub fn add_layer<L>(&mut self, layer: L) -> &mut Self
    where
        L: Layer<tracing_subscriber::Registry> + Send + Sync + 'static,
    {
        self.layers.push(Box::new(layer));
        self
    }

    /// Add a scheduled task
    pub fn add_scheduler<T>(&mut self, scheduler: T) -> &mut Self
    where
        T: FnOnce(Arc<App>) -> Box<dyn Future<Output = Result<String>> + Send> + 'static,
    {
        self.schedulers.push(Box::new(scheduler));
        self
    }

    /// Add a shutdown hook
    pub fn add_shutdown_hook<T>(&mut self, hook: T) -> &mut Self
    where
        T: FnOnce(Arc<App>) -> Box<dyn Future<Output = Result<String>> + Send> + 'static,
    {
        self.shutdown_hooks.push(Box::new(hook));
        self
    }

    /// Running
    pub async fn run(&mut self) {
        match self.inner_run().await {
            Err(e) => {
                log::error!("{:?}", e);
            }
            Ok(_app) => { /* return? */ }
        }
    }

    async fn inner_run(&mut self) -> Result<Arc<App>> {
        // 1. load toml config
        self.config = TomlConfigRegistry::new(&self.config_path, self.env)?;

        // 2. build plugin
        self.build_plugins().await;

        // 3. service dependency inject
        service::auto_inject_service(self)?;

        // 4. schedule
        self.schedule().await
    }

    async fn build_plugins(&mut self) {
        LogPlugin.immediately_build(self);

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
                    log::info!("{} plugin registered", plugin.name());
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

        let schedulers = std::mem::take(&mut self.schedulers);
        for task in schedulers {
            let poll_future = task(app.clone());
            let poll_future = Box::into_pin(poll_future);
            match tokio::spawn(poll_future).await? {
                Err(e) => log::error!("{}", e),
                Ok(msg) => log::info!("scheduled result: {}", msg),
            }
        }

        // FILO: The hooks added by the plugin built first should be executed later
        while let Some(hook) = self.shutdown_hooks.pop() {
            let result = Box::into_pin(hook(app.clone())).await?;
            log::info!("shutdown result: {result}");
        }
        Ok(app)
    }

    fn build_app(&mut self) -> Arc<App> {
        let components = std::mem::take(&mut self.components);
        let config = std::mem::take(&mut self.config);
        Arc::new(App { components, config })
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            env: Env::init(),
            config_path: Path::new("./config/app.toml").to_path_buf(),
            layers: Default::default(),
            plugin_registry: Default::default(),
            config: Default::default(),
            components: Default::default(),
            schedulers: Default::default(),
            shutdown_hooks: Default::default(),
        }
    }
}

impl ConfigRegistry for App {
    fn get_config<T>(&self) -> Result<T>
    where
        T: serde::de::DeserializeOwned + crate::config::Configurable,
    {
        self.config.get_config::<T>()
    }
}

impl ConfigRegistry for AppBuilder {
    fn get_config<T>(&self) -> Result<T>
    where
        T: serde::de::DeserializeOwned + crate::config::Configurable,
    {
        self.config.get_config::<T>()
    }
}
