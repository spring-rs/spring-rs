use apalis::prelude::Monitor;
use spring::{
    app::AppBuilder,
    async_trait,
    error::Result,
    plugin::{component::ComponentRef, ComponentRegistry, MutableComponentRegistry, Plugin},
    tracing,
};

pub use apalis;
#[cfg(feature = "redis")]
pub use apalis_redis;
#[cfg(any(
    feature = "sql-postgres",
    feature = "sql-sqlite",
    feature = "sql-mysql"
))]
pub use apalis_sql;

pub struct ApalisPlugin;

pub type WorkerRegister = fn(&mut AppBuilder, Monitor) -> Monitor;

#[async_trait]
impl Plugin for ApalisPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        if let Some(builders) = app.get_component::<Vec<WorkerRegister>>() {
            let mut monitor = Monitor::new();
            for build_fn in &builders {
                monitor = build_fn(app, monitor);
            }
            if builders.len() > 0 {
                app.add_scheduler(move |_app| Box::new(Self::schedule(monitor)));
            }
        }
    }
}

impl ApalisPlugin {
    async fn schedule(monitor: Monitor) -> Result<String> {
        let _ = monitor.run_with_signal(shutdown_signal()).await;
        Ok("apalis scheduled finished".to_string())
    }
}

async fn shutdown_signal() -> std::io::Result<()> {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    Ok(tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal, waiting for apalis shutdown")
        },
        _ = terminate => {
            tracing::info!("Received kill signal, waiting for apalis shutdown")
        },
    })
}

pub trait ApalisConfigurator {
    fn add_worker(&mut self, worker_register: WorkerRegister) -> &mut Self;
}

impl ApalisConfigurator for AppBuilder {
    fn add_worker(&mut self, worker_register: WorkerRegister) -> &mut Self {
        if let Some(workers) = self.get_component_ref::<Vec<WorkerRegister>>() {
            unsafe {
                let raw_ptr = ComponentRef::into_raw(workers);
                let workers = &mut *(raw_ptr as *mut Vec<WorkerRegister>);
                workers.push(worker_register);
            }
            self
        } else {
            self.add_component(vec![worker_register])
        }
    }
}
