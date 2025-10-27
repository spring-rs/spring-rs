use crate::{
    handler::{BoxedHandler, Handler},
    JobId, JobScheduler,
};
use anyhow::Context;
use spring::app::App;
use std::{sync::Arc, time::Duration};

#[derive(Clone)]
enum Trigger {
    OneShot(u64),
    FixedDelay(u64),
    FixedRate(u64),
    Cron(String),
}

#[derive(Clone)]
pub struct Job {
    trigger: Trigger,
    handler: BoxedHandler,
}

pub struct JobBuilder {
    trigger: Trigger,
}

impl Job {
    pub fn one_shot(delay_seconds: u64) -> JobBuilder {
        JobBuilder {
            trigger: Trigger::OneShot(delay_seconds),
        }
    }
    /// TODO: tokio-cron-scheduler not support: <https://github.com/mvniekerk/tokio-cron-scheduler/issues/56>
    pub fn fix_delay(seconds: u64) -> JobBuilder {
        JobBuilder {
            trigger: Trigger::FixedDelay(seconds),
        }
    }
    pub fn fix_rate(seconds: u64) -> JobBuilder {
        JobBuilder {
            trigger: Trigger::FixedRate(seconds),
        }
    }
    pub fn cron(cron: &str) -> JobBuilder {
        JobBuilder {
            trigger: Trigger::Cron(cron.to_string()),
        }
    }
    pub fn build(self, app: Arc<App>) -> tokio_cron_scheduler::Job {
        let handler = self.handler;
        match self.trigger {
            Trigger::OneShot(seconds) => tokio_cron_scheduler::Job::new_one_shot_async(
                Duration::from_secs(seconds),
                move |job_id, jobs| {
                    let handler = handler.clone();
                    let app = app.clone();
                    Box::pin(async move { handler.call(job_id, jobs, app).await })
                },
            ),
            // TODO
            Trigger::FixedDelay(seconds) => tokio_cron_scheduler::Job::new_repeated_async(
                Duration::from_secs(seconds),
                move |job_id, jobs| {
                    Box::pin(Self::call(handler.clone(), job_id, jobs, app.clone()))
                },
            ),
            Trigger::FixedRate(seconds) => tokio_cron_scheduler::Job::new_repeated_async(
                Duration::from_secs(seconds),
                move |job_id, jobs| {
                    Box::pin(Self::call(handler.clone(), job_id, jobs, app.clone()))
                },
            ),
            Trigger::Cron(schedule) => tokio_cron_scheduler::Job::new_async_tz(
                schedule.as_str(),
                chrono::Local,
                move |job_id, jobs| {
                    Box::pin(Self::call(handler.clone(), job_id, jobs, app.clone()))
                },
            ),
        }
        .context("build job failed")
        .unwrap()
    }

    async fn call(handler: BoxedHandler, job_id: JobId, jobs: JobScheduler, app: Arc<App>) {
        handler.call(job_id, jobs, app).await
    }
}

impl JobBuilder {
    pub fn run<H, A>(self, handler: H) -> Job
    where
        H: Handler<A> + Sync,
        A: 'static,
    {
        Job {
            trigger: self.trigger,
            handler: BoxedHandler::from_handler(handler),
        }
    }
}
