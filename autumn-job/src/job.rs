use std::{sync::Arc, time::Duration};

use anyhow::Context;
use autumn_boot::app::App;

use crate::handler::{BoxedHandler, Handler};

#[derive(Clone)]
enum Trigger {
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
    /// TODO: tokio-cron-scheduler not support
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
    pub(crate) fn build(self, app: Arc<App>) -> tokio_cron_scheduler::Job {
        let handler = self.handler;
        match self.trigger {
            // TODO
            Trigger::FixedDelay(seconds) => tokio_cron_scheduler::Job::new_repeated_async(
                Duration::from_secs(seconds),
                move |job_id, jobs| {
                    let handler = handler.clone();
                    let app = app.clone();
                    Box::pin(async move { handler.call(job_id, jobs, app).await })
                },
            ),
            Trigger::FixedRate(seconds) => tokio_cron_scheduler::Job::new_repeated_async(
                Duration::from_secs(seconds),
                move |job_id, jobs| {
                    let handler = handler.clone();
                    let app = app.clone();
                    Box::pin(async move { handler.call(job_id, jobs, app).await })
                },
            ),
            Trigger::Cron(schedule) => {
                tokio_cron_scheduler::Job::new_async(schedule.as_str(), move |job_id, jobs| {
                    let handler = handler.clone();
                    let app = app.clone();
                    Box::pin(async move { handler.call(job_id, jobs, app).await })
                })
            }
        }
        .context("build job failed")
        .unwrap()
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
