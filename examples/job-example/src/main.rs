use anyhow::Context;
use spring::App;
use spring::{cron, fix_delay, fix_rate};
use spring_job::{extractor::Component, handler::TypedJob, JobConfigurator, JobPlugin, Jobs};
use spring_sqlx::{
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
}

#[cron("1/10 * * * * *")]
async fn cron_job(Component(db): Component<ConnectPool>) {
    let time: String = sqlx::query("select TO_CHAR(now(),'YYYY-MM-DD HH24:MI:SS') as time")
        .fetch_one(&db)
        .await
        .context("query failed")
        .unwrap()
        .get("time");
    println!("cron scheduled: {:?}", time)
}

#[fix_delay(5)]
async fn fix_delay_job() {
    let now = SystemTime::now();
    let datetime: sqlx::types::chrono::DateTime<sqlx::types::chrono::Local> = now.into();
    let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S");
    println!("fix delay scheduled: {}", formatted_time)
}

#[fix_rate(5)]
async fn fix_rate_job() {
    tokio::time::sleep(Duration::from_secs(10)).await;
    let now = SystemTime::now();
    let datetime: sqlx::types::chrono::DateTime<sqlx::types::chrono::Local> = now.into();
    let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S");
    println!("fix rate scheduled: {}", formatted_time)
}
