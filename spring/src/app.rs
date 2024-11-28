use crate::banner;
use crate::config::env::Env;
use crate::config::toml::TomlConfigRegistry;
use crate::config::ConfigRegistry;
use crate::log::{BoxLayer, LogPlugin};
use crate::plugin::component::ComponentRef;
use crate::plugin::{service, ComponentRegistry, MutableComponentRegistry, Plugin};
use crate::{
    error::Result,
    plugin::{component::DynComponentRef, PluginRef},
};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::any::Any;
use std::str::FromStr;
use std::sync::RwLock;
use std::{any, collections::HashSet, future::Future, path::Path, sync::Arc};
use tracing_subscriber::Layer;

type Registry<T> = DashMap<String, T>;
type Scheduler<T> = dyn FnOnce(Arc<App>) -> Box<dyn Future<Output = Result<T>> + Send>;

/// Running Applications
#[derive(Default)]
pub struct App {
    env: Env,
    /// Component
    components: Registry<DynComponentRef>,
    config: TomlConfigRegistry,
}

/// AppBuilder: Application under construction
/// The application consists of three important parts:
/// - Plugin management
/// - Component management
/// - Configuration management
pub struct AppBuilder {
    pub(crate) env: Env,
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
    /// Preparing to build the application
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> AppBuilder {
        AppBuilder::default()
    }

    /// Currently active environment
    /// * [Env]
    pub fn get_env(&self) -> Env {
        self.env
    }

    /// Returns an instance of the currently configured global [`App`].
    ///
    /// **NOTE**: This global App is initialized after the application is built,
    /// please use it when the app is running, don't use it during the build process,
    /// such as during the plug-in build process.
    pub fn global() -> Arc<App> {
        GLOBAL_APP
            .read()
            .expect("GLOBAL_APP RwLock poisoned")
            .clone()
    }

    fn set_global(app: Arc<App>) {
        let mut global_app = GLOBAL_APP.write().expect("GLOBAL_APP RwLock poisoned");
        *global_app = app;
    }
}

static GLOBAL_APP: Lazy<RwLock<Arc<App>>> = Lazy::new(|| RwLock::new(Arc::new(App::default())));

unsafe impl Send for AppBuilder {}
unsafe impl Sync for AppBuilder {}

impl AppBuilder {
    /// Currently active environment
    /// * [Env]
    #[inline]
    pub fn get_env(&self) -> Env {
        self.env
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
    #[inline]
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
    pub fn use_config_file(&mut self, config_path: &str) -> &mut Self {
        self.config = TomlConfigRegistry::new(Path::new(config_path), self.env)
            .expect("config file load failed");
        self
    }

    /// Use an existing toml string to configure the application.
    /// For example, use include_str!('app.toml') to compile the file into the program.
    ///
    /// **Note**: This configuration method only supports one configuration content and does not support multiple environments.
    pub fn use_config_str(&mut self, toml_content: &str) -> &mut Self {
        self.config =
            TomlConfigRegistry::from_str(toml_content).expect("config content parse failed");
        self
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

    /// The `run` method is suitable for applications that contain scheduling logic,
    /// such as web, job, and stream.
    ///
    /// * [spring-web](https://docs.rs/spring-web)
    /// * [spring-job](https://docs.rs/spring-job)
    /// * [spring-stream](https://docs.rs/spring-stream)
    pub async fn run(&mut self) {
        match self.inner_run().await {
            Err(e) => {
                log::error!("{:?}", e);
            }
            Ok(app) => App::set_global(app),
        }
    }

    async fn inner_run(&mut self) -> Result<Arc<App>> {
        // 1. load toml config
        self.load_config_if_need()?;

        banner::print_banner(self);

        // 2. build plugin
        self.build_plugins().await;

        // 3. service dependency inject
        service::auto_inject_service(self)?;

        // 4. schedule
        self.schedule().await
    }

    /// Unlike the [`run`] method, the `build` method is suitable for applications that do not contain scheduling logic.
    /// This method returns the built App, and developers can implement logic such as command lines and task scheduling by themselves.
    pub async fn build(&mut self) -> Result<Arc<App>> {
        // 1. load toml config
        self.load_config_if_need()?;

        // 2. build plugin
        self.build_plugins().await;

        // 3. service dependency inject
        service::auto_inject_service(self)?;

        Ok(self.build_app())
    }

    fn load_config_if_need(&mut self) -> Result<()> {
        if self.config.is_empty() {
            self.config = TomlConfigRegistry::new(Path::new("./config/app.toml"), self.env)?;
        }
        Ok(())
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
        let mut handles = vec![];
        for task in schedulers {
            let poll_future = task(app.clone());
            let poll_future = Box::into_pin(poll_future);
            handles.push(tokio::spawn(poll_future));
        }

        while let Some(handle) = handles.pop() {
            match handle.await? {
                Err(e) => log::error!("{:?}", e),
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
        Arc::new(App {
            env: self.env,
            components,
            config,
        })
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            env: Env::init(),
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

macro_rules! impl_component_registry {
    ($ty:ident) => {
        impl ComponentRegistry for $ty {
            /// Get the component reference of the specified type
            fn get_component_ref<T>(&self) -> Option<ComponentRef<T>>
            where
                T: Any + Send + Sync,
            {
                let component_name = std::any::type_name::<T>();
                let pair = self.components.get(component_name)?;
                let component_ref = pair.value().clone();
                component_ref.downcast::<T>()
            }

            /// Get the component of the specified type
            fn get_component<T>(&self) -> Option<T>
            where
                T: Clone + Send + Sync + 'static,
            {
                let component_ref = self.get_component_ref();
                component_ref.map(|c| T::clone(&c))
            }

            /// Get all built components. The return value is the full crate path of all components
            fn get_components(&self) -> Vec<String> {
                self.components.iter().map(|e| e.key().clone()).collect()
            }
        }
    };
}

impl_component_registry!(App);
impl_component_registry!(AppBuilder);

impl MutableComponentRegistry for AppBuilder {
    /// Add component to the registry
    fn add_component<T>(&mut self, component: T) -> &mut Self
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
}
