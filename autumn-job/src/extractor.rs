use async_trait::async_trait;
use autumn_boot::app::App;
use uuid::Uuid;

use crate::JobScheduler;

#[async_trait]
pub trait FromApp {
    async fn from_app(job_id: &Uuid, jobs: &JobScheduler, app: &App) -> Self;
}
