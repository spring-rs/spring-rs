use crate::{JobId, JobScheduler};
use serde::de::DeserializeOwned;
use spring::async_trait;
use spring::config::ConfigRegistry;
use spring::plugin::ComponentRegistry;
use spring::{app::App, config::Configurable};
use std::ops::{Deref, DerefMut};

#[async_trait]
pub trait FromApp {
    async fn from_app(job_id: &JobId, scheduler: &JobScheduler, app: &App) -> Self;
}

pub struct Component<T: Clone>(pub T);

#[async_trait]
impl<T> FromApp for Component<T>
where
    T: Clone + Send + Sync + 'static,
{
    async fn from_app(_job_id: &JobId, _scheduler: &JobScheduler, app: &App) -> Self {
        match app.get_component_ref::<T>() {
            Some(component) => Component(T::clone(&component)),
            None => panic!(
                "There is no component of `{}` type",
                std::any::type_name::<T>()
            ),
        }
    }
}

impl<T: Clone> Deref for Component<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> DerefMut for Component<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait]
impl FromApp for JobId {
    async fn from_app(job_id: &JobId, _scheduler: &JobScheduler, _app: &App) -> Self {
        *job_id
    }
}

#[async_trait]
impl FromApp for JobScheduler {
    async fn from_app(_job_id: &JobId, scheduler: &JobScheduler, _app: &App) -> Self {
        scheduler.clone()
    }
}

pub struct Data<T: DeserializeOwned>(Option<T>);

#[async_trait]
impl<T: DeserializeOwned> FromApp for Data<T> {
    async fn from_app(job_id: &JobId, scheduler: &JobScheduler, _app: &App) -> Self {
        let mut guard = scheduler.context.metadata_storage.write().await;
        let job = guard.get(*job_id).await.expect("job get failed");
        Self(job.map(|j| serde_json::from_slice(&j.extra).expect("job extra parse to json failed")))
    }
}

pub struct Config<T>(pub T)
where
    T: serde::de::DeserializeOwned + Configurable;

#[async_trait]
impl<T> FromApp for Config<T>
where
    T: serde::de::DeserializeOwned + Configurable,
{
    async fn from_app(_job_id: &JobId, _scheduler: &JobScheduler, app: &App) -> Self {
        match app.get_config::<T>() {
            Ok(config) => Config(config),
            Err(e) => panic!(
                "get config for typeof {} failed: {}",
                std::any::type_name::<T>(),
                e
            ),
        }
    }
}

impl<T> Deref for Config<T>
where
    T: serde::de::DeserializeOwned + Configurable,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
