use anyhow::Context;
use spring::{auto_config, App};
use spring_job::job::Job;
use spring_job::{cron, fix_delay, fix_rate, JobScheduler};
use spring_job::{extractor::Component, JobConfigurator, JobPlugin};
use spring_sqlx::{
    sqlx::{self, Row},
    ConnectPool, SqlxPlugin,
};
use spring_web::axum::response::IntoResponse;
use spring_web::extractor::AppRef;
use spring_web::{
    error::Result, extractor::Component as WebComponent, get, WebConfigurator, WebPlugin,
};
use std::time::{Duration, SystemTime};

#[auto_config(JobConfigurator, WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(JobPlugin)
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await;
}

#[get("/")]
async fn new_job(
    WebComponent(sched): WebComponent<JobScheduler>,
    AppRef(app): AppRef,
) -> Result<impl IntoResponse> {
    sched
        .add(Job::one_shot(3).run(after_3s_job).build(app))
        .await
        .context("register job failed")?;
    Ok("ok")
}

async fn after_3s_job() {
    let now = SystemTime::now();
    let datetime: sqlx::types::chrono::DateTime<sqlx::types::chrono::Local> = now.into();
    let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S");
    println!("one shot scheduled: {:?}", formatted_time)
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
