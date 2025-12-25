use serde::{Deserialize, Serialize};
use spring::{
    app::AppBuilder,
    auto_config,
    plugin::{ComponentRegistry, MutableComponentRegistry},
    tracing, App,
};
use spring_apalis::apalis::{
    layers::{ErrorHandlingLayer, WorkerBuilderExt as _},
    prelude::{Context, Monitor, Worker, WorkerBuilder, WorkerFactoryFn as _},
};
use spring_apalis::apalis_redis::RedisStorage;
use spring_apalis::{apalis::prelude::*, ApalisConfigurator as _, ApalisPlugin};
use spring_redis::{Redis, RedisPlugin};
use spring_web::{
    axum::response::IntoResponse, extractor::Component, get, middleware::ServiceExt,
    WebConfigurator, WebPlugin,
};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct LongRunningJob {}

async fn long_running_task(_task: LongRunningJob, worker: Worker<Context>) {
    loop {
        tracing::info!("is_shutting_down: {}", worker.is_shutting_down());
        if worker.is_shutting_down() {
            tracing::info!("saving the job state");
            break;
        }
        tokio::time::sleep(Duration::from_secs(3)).await; // Do some hard thing
    }
    tracing::info!("Shutdown complete!");
}

fn long_running_task_register(app: &mut AppBuilder, monitor: Monitor) -> Monitor {
    let redis = app.get_expect_component::<Redis>();
    let storage = RedisStorage::new(redis);
    app.add_component(storage.clone());

    let worker = WorkerBuilder::new("long_running")
        .catch_panic()
        .enable_tracing()
        .rate_limit(5, Duration::from_secs(1))
        .concurrency(2)
        .backend(storage)
        .build_fn(long_running_task);

    monitor.register(worker)
}

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(RedisPlugin)
        .add_plugin(WebPlugin)
        .add_plugin(ApalisPlugin)
        .add_worker(long_running_task_register)
        .run()
        .await
}

#[get("/")]
pub async fn start_job(
    Component(mut storage): Component<RedisStorage<LongRunningJob>>,
) -> impl IntoResponse {
    storage.push(LongRunningJob {}).await.unwrap();
}
