use anyhow::Context;
use autumn::App;
use autumn::{cron, fix_delay, fix_rate};
use autumn_job::{extractor::Component, handler::TypedJob, JobConfigurator, JobPlugin, Jobs};
use autumn_sqlx::{
    sqlx::{self, Row},
    ConnectPool, SqlxPlugin,
};
use std::time::{Duration, SystemTime};

#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(JobPlugin)
        .add_plugin(SqlxPlugin)
        .add_jobs(jobs())
        .run()
        .await;

    tokio::time::sleep(Duration::from_secs(100)).await;
}

fn jobs() -> Jobs {
    Jobs::new()
        .typed_job(cron_job)
        .typed_job(fix_delay_job)
        .typed_job(fix_rate_job)
        .to_owned()
}

#[cron("1/10 * * * * *")]
async fn cron_job(Component(db): Component<ConnectPool>) {
    let time: String = sqlx::query("select now() as time")
        .fetch_one(&db)
        .await
        .context("query failed")
        .unwrap()
        .get("time");
    println!("cron scheduled: {:?}", time)
}

#[fix_delay(5)]
async fn fix_delay_job() {
    println!("fix delay scheduled: {:?}", SystemTime::now())
}

#[fix_rate(5)]
async fn fix_rate_job() {
    tokio::time::sleep(Duration::from_secs(10)).await;
    println!("fix rate scheduled: {:?}", SystemTime::now())
}
