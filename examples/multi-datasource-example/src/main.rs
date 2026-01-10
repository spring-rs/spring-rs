//! Multi-datasource Example
//!
//! This example demonstrates how to configure and use multiple database connections
//! in a spring-rs application. This is useful for:
//! - Read/Write splitting (primary for writes, secondary for reads)
//! - Connecting to multiple different databases
//! - Database sharding scenarios
//!
//! # Usage
//!
//! Set the environment variables before running:
//! ```bash
//! export PRIMARY_DATABASE_URL="postgres://user:pass@localhost:5432/primary_db"
//! export SECONDARY_DATABASE_URL="postgres://user:pass@localhost:5432/secondary_db"
//! cargo run -p multi-datasource-example
//! ```

mod multi_datasource;

use anyhow::Context;
use multi_datasource::{MultiDatasourcePlugin, PrimaryDb, SecondaryDb};
use sea_orm::{ConnectionTrait, Statement};
use spring::{auto_config, App};
use spring_web::get;
use spring_web::{
    axum::response::IntoResponse, error::Result, extractor::Component, WebConfigurator, WebPlugin,
};

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(MultiDatasourcePlugin)
        .add_plugin(WebPlugin)
        .run()
        .await
}

/// Example handler that reads from the primary database
#[get("/primary")]
async fn query_primary(Component(db): Component<PrimaryDb>) -> Result<impl IntoResponse> {
    // Example: Execute a simple query on the primary database
    let result = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT 'Hello from Primary DB' as message".to_string(),
        ))
        .await
        .context("query primary database failed")?;

    let message = result
        .map(|row| row.try_get::<String>("", "message").unwrap_or_default())
        .unwrap_or_else(|| "No result".to_string());

    Ok(message)
}

/// Example handler that reads from the secondary database
#[get("/secondary")]
async fn query_secondary(Component(db): Component<SecondaryDb>) -> Result<impl IntoResponse> {
    // Example: Execute a simple query on the secondary database
    let result = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT 'Hello from Secondary DB' as message".to_string(),
        ))
        .await
        .context("query secondary database failed")?;

    let message = result
        .map(|row| row.try_get::<String>("", "message").unwrap_or_default())
        .unwrap_or_else(|| "No result".to_string());

    Ok(message)
}

/// Example handler that uses both databases
/// This demonstrates a common read/write splitting pattern:
/// - Write operations go to primary
/// - Read operations go to secondary
#[get("/both")]
async fn query_both(
    Component(primary): Component<PrimaryDb>,
    Component(secondary): Component<SecondaryDb>,
) -> Result<impl IntoResponse> {
    // Query primary database
    let primary_result = primary
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT current_database() as db_name".to_string(),
        ))
        .await
        .context("query primary database failed")?;

    let primary_db = primary_result
        .map(|row| row.try_get::<String>("", "db_name").unwrap_or_default())
        .unwrap_or_else(|| "unknown".to_string());

    // Query secondary database
    let secondary_result = secondary
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT current_database() as db_name".to_string(),
        ))
        .await
        .context("query secondary database failed")?;

    let secondary_db = secondary_result
        .map(|row| row.try_get::<String>("", "db_name").unwrap_or_default())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(format!(
        "Primary DB: {primary_db}, Secondary DB: {secondary_db}"
    ))
}
