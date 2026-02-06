use serde::Serialize;
use spring::{auto_config, App};

use spring_diesel_orm::diesel_async::{DieselAsyncOrmPlugin, SqliteDeadPoolConnectionPool};

use spring_web::get;
use spring_web::{
    axum::response::{IntoResponse, Json},
    error::Result,
    extractor::{Component},
    WebConfigurator, WebPlugin,
};

use anyhow::Context;

use diesel::prelude::*;
use crate::schema::users;
use diesel_async::{RunQueryDsl};


// ordinary diesel model setup
pub mod schema;

#[allow(dead_code)]
#[derive(Debug, Serialize, Queryable, Identifiable, Selectable, QueryableByName)]
#[diesel(table_name = users)]
struct User {
    id: i64,
    name: String,
    active: bool,
}

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(DieselAsyncOrmPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await;
}

#[get("/users")]
async fn get_users(Component(db): Component<SqliteDeadPoolConnectionPool>) -> Result<impl IntoResponse> {
    let mut connection = db.get().await.context("failed to get sqlite connection")?;
    
    //.context("failed to get db connection")?;
    //let connection = ;
    let rows: Vec<User> = users::table
        .filter(users::active.eq(true))
        .limit(10)
        .load(connection.as_mut())
        .await
        .context("query users failed")?;
    //.await.context("query users failed")?;
    Ok(Json(rows))
}
