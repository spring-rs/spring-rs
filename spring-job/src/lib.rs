pub mod extractor;
pub mod handler;
pub mod job;

use anyhow::Context;
use async_trait::async_trait;
use spring_boot::error::Result;
use spring_boot::{
    app::{App, AppBuilder},
    plugin::Plugin,
};
use job::Job;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct Jobs(Vec<Job>);

impl Jobs {
    pub fn new() -> Self {
        Self::default()
    }
    fn single(job: Job) -> Self {
        Self(vec![job])
    }
}

impl JobConfigurator for Jobs {
    fn add_job(&mut self, job: Job) -> &mut Self {
        self.0.push(job);
        self
    }
    fn add_jobs(&mut self, jobs: Jobs) -> &mut Self {
        for job in jobs.0 {
            self.0.push(job);
        }
        self
    }
}

impl Deref for Jobs {
    type Target = Vec<Job>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type JobScheduler = tokio_cron_scheduler::JobScheduler;

pub trait JobConfigurator {
    fn add_job(&mut self, job: Job) -> &mut Self;
    fn add_jobs(&mut self, job: Jobs) -> &mut Self;
}

impl JobConfigurator for AppBuilder {
    fn add_job(&mut self, job: Job) -> &mut Self {
        if let Some(jobs) = self.get_component::<Jobs>() {
            unsafe {
                let raw_ptr = Arc::into_raw(jobs);
                let jobs = &mut *(raw_ptr as *mut Vec<Job>);
                jobs.push(job);
            }
            self
        } else {
            self.add_component(Jobs::single(job))
        }
    }

    fn add_jobs(&mut self, new_jobs: Jobs) -> &mut Self {
        if let Some(jobs) = self.get_component::<Jobs>() {
            unsafe {
                let raw_ptr = Arc::into_raw(jobs);
                let jobs = &mut *(raw_ptr as *mut Jobs);
                jobs.add_jobs(new_jobs);
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
        app.add_scheduler(|app: Arc<App>| Box::new(Self::schedule(app)));
    }

    fn config_prefix(&self) -> &str {
        "job"
    }
}

impl JobPlugin {
    async fn schedule(app: Arc<App>) -> Result<String> {
        let jobs = app.get_component::<Jobs>();

        if jobs.is_none() {
            let msg = "No tasks are registered, so the task scheduler does not start.";
            tracing::info!(msg);
            return Ok(msg.to_string());
        }

        let jobs = jobs.unwrap();

        let mut sched = JobScheduler::new().await.context("job init failed")?;

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
                println!("Shut down done");
            })
        }));

        // Start the scheduler
        sched.start().await.context("job scheduler start failed")?;

        Ok("job schedule finished".to_string())
    }
}
