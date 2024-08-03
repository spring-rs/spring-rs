use anyhow::Context;
use autumn_boot::app::App;
use autumn_job::{extractor::Component, job::Job, JobConfigurator, JobPlugin};
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
        .add_job(Job::cron("1/10 * * * * *").run(cron_job))
        .add_job(Job::fix_delay(6).run(fix_delay_job))
        .add_job(Job::fix_rate(6).run(fix_rate_job))
        .run()
        .await;

    tokio::time::sleep(Duration::from_secs(100)).await;
}

async fn cron_job(Component(db): Component<ConnectPool>) {
    let time: String = sqlx::query("select now() as time")
        .fetch_one(&db)
        .await
        .context("query failed")
        .unwrap()
        .get("time");
    println!("cron scheduled: {:?}", time)
}

async fn fix_delay_job() {
    println!("fix delay scheduled: {:?}", SystemTime::now())
}

async fn fix_rate_job() {
    tokio::time::sleep(Duration::from_secs(10)).await;
    println!("fix rate scheduled: {:?}", SystemTime::now())
}
