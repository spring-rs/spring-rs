use async_trait::async_trait;
use autumn_boot::app::App;
use uuid::Uuid;

use crate::JobScheduler;

#[async_trait]
pub trait FromApp {
    async fn from_app(job_id: &Uuid, jobs: &JobScheduler, app: &App) -> Self;
}

pub struct Component<T>(pub T);

#[async_trait]
impl<T> FromApp for Component<T>
where
    T: Clone + Send + Sync + 'static,
{
    async fn from_app(_job_id: &Uuid, _jobs: &JobScheduler, app: &App) -> Self {
        match app.get_component::<T>() {
            Some(component) => Component(T::clone(&component)),
            None => panic!(
                "There is no component of `{}` type",
                std::any::type_name::<T>()
            ),
        }
    }
}

pub struct JobId(pub Uuid);

#[async_trait]
impl FromApp for JobId {
    async fn from_app(job_id: &Uuid, _jobs: &JobScheduler, _app: &App) -> Self {
        JobId(job_id.clone())
    }
}

pub struct Jobs(pub JobScheduler);

#[async_trait]
impl FromApp for Jobs {
    async fn from_app(_job_id: &Uuid, jobs: &JobScheduler, _app: &App) -> Self {
        Jobs(jobs.clone())
    }
}
