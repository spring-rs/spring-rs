use async_trait::async_trait;
use spring_boot::app::App;
use std::ops::{Deref, DerefMut};

use crate::{JobId, JobScheduler};

#[async_trait]
pub trait FromApp {
    async fn from_app(job_id: &JobId, scheduler: &JobScheduler, app: &App) -> Self;
}

pub struct Component<T>(pub T);

#[async_trait]
impl<T> FromApp for Component<T>
where
    T: Clone + Send + Sync + 'static,
{
    async fn from_app(_job_id: &JobId, _scheduler: &JobScheduler, app: &App) -> Self {
        match app.get_component::<T>() {
            Some(component) => Component(T::clone(&component)),
            None => panic!(
                "There is no component of `{}` type",
                std::any::type_name::<T>()
            ),
        }
    }
}

impl<T> Deref for Component<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Component<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait]
impl FromApp for JobId {
    async fn from_app(job_id: &JobId, _scheduler: &JobScheduler, _app: &App) -> Self {
        job_id.clone()
    }
}

#[async_trait]
impl FromApp for JobScheduler {
    async fn from_app(_job_id: &JobId, scheduler: &JobScheduler, _app: &App) -> Self {
        scheduler.clone()
    }
}
