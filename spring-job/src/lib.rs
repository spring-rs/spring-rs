//! [![spring-rs](https://img.shields.io/github/stars/spring-rs/spring-rs)](https://spring-rs.github.io/docs/plugins/spring-job)
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]

pub mod extractor;
pub mod handler;
pub mod job;

/////////////////job-macros/////////////////////
/// To use these Procedural Macros, you need to add `spring-job` dependency
pub use spring_macros::cron;
pub use spring_macros::fix_delay;
pub use spring_macros::fix_rate;
pub use spring_macros::one_shot;

use anyhow::Context;
use job::Job;
use spring::async_trait;
use spring::error::Result;
use spring::plugin::component::ComponentRef;
use spring::plugin::ComponentRegistry;
use spring::plugin::MutableComponentRegistry;
use spring::{
    app::{App, AppBuilder},
    plugin::Plugin,
};
use std::ops::Deref;
use std::sync::Arc;
use tokio_cron_scheduler::JobSchedulerError;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct Jobs(Vec<Job>);

impl Jobs {
    pub fn new() -> Self {
        Self::default()
    }
    fn single(job: Job) -> Self {
        Self(vec![job])
    }

    pub fn add_job(mut self, job: Job) -> Self {
        self.0.push(job);
        self
    }

    pub fn add_jobs(mut self, jobs: Jobs) -> Self {
        for job in jobs.0 {
            self.0.push(job);
        }
        self
    }

    fn merge(&mut self, jobs: Jobs) {
        for job in jobs.0 {
            self.0.push(job);
        }
    }
}

impl Deref for Jobs {
    type Target = Vec<Job>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type JobId = Uuid;
pub type JobScheduler = tokio_cron_scheduler::JobScheduler;

pub trait JobConfigurator {
    fn add_job(&mut self, job: Job) -> &mut Self;
    fn add_jobs(&mut self, job: Jobs) -> &mut Self;
}

impl JobConfigurator for AppBuilder {
    fn add_job(&mut self, job: Job) -> &mut Self {
        if let Some(jobs) = self.get_component_ref::<Jobs>() {
            unsafe {
                let raw_ptr = ComponentRef::into_raw(jobs);
                let jobs = &mut *(raw_ptr as *mut Vec<Job>);
                jobs.push(job);
            }
            self
        } else {
            self.add_component(Jobs::single(job))
        }
    }

    fn add_jobs(&mut self, new_jobs: Jobs) -> &mut Self {
        if let Some(jobs) = self.get_component_ref::<Jobs>() {
            unsafe {
                let raw_ptr = ComponentRef::into_raw(jobs);
                let jobs = &mut *(raw_ptr as *mut Jobs);
                jobs.merge(new_jobs);
            }
            self
        } else {
            self.add_component(new_jobs)
        }
    }
}

pub struct JobPlugin;

#[async_trait]
impl Plugin for JobPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let sched = Self::new_scheduler().await.expect("build scheduler failed");
        app.add_component(sched)
            .add_scheduler(|app: Arc<App>| Box::new(Self::schedule(app)));
    }
}

impl JobPlugin {
    async fn new_scheduler() -> std::result::Result<JobScheduler, JobSchedulerError> {
        #[cfg(feature = "postgres_storage")]
        {
            let metadata_storage = Box::new(tokio_cron_scheduler::PostgresMetadataStore::default());
            let notification_storage =
                Box::new(tokio_cron_scheduler::PostgresNotificationStore::default());
            let job_code = Box::new(tokio_cron_scheduler::SimpleJobCode::default());
            let notification_code =
                Box::new(tokio_cron_scheduler::SimpleNotificationCode::default());
            JobScheduler::new_with_storage_and_code(
                metadata_storage,
                notification_storage,
                job_code,
                notification_code,
                200,
            )
            .await
        }
        #[cfg(all(not(feature = "postgres_storage"), feature = "nats_storage"))]
        {
            let metadata_storage = Box::new(tokio_cron_scheduler::NatsMetadataStore::default());
            let notification_storage =
                Box::new(tokio_cron_scheduler::NatsNotificationStore::default());
            let job_code = Box::new(tokio_cron_scheduler::SimpleJobCode::default());
            let notification_code =
                Box::new(tokio_cron_scheduler::SimpleNotificationCode::default());
            JobScheduler::new_with_storage_and_code(
                metadata_storage,
                notification_storage,
                job_code,
                notification_code,
                200,
            )
            .await
        }
        #[cfg(all(not(feature = "postgres_storage"), not(feature = "nats_storage")))]
        {
            JobScheduler::new().await
        }
    }

    async fn schedule(app: Arc<App>) -> Result<String> {
        let mut sched = app.get_expect_component::<JobScheduler>();
        let jobs = app.get_component_ref::<Jobs>();

        let jobs = match jobs {
            None => {
                let msg = "No tasks are registered, so the task scheduler does not start.";
                tracing::info!(msg);
                return Ok(msg.to_string());
            }
            Some(jobs) => jobs,
        };

        for job in jobs.deref().iter() {
            sched
                .add(job.to_owned().build(app.clone()))
                .await
                .context("add job failed")?;
        }

        sched.shutdown_on_ctrl_c();

        // Add code to be run during/after shutdown
        sched.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                tracing::info!("Shut down done");
            })
        }));

        // Start the scheduler
        sched.start().await.context("job scheduler start failed")?;

        Ok("job schedule started".to_string())
    }
}
