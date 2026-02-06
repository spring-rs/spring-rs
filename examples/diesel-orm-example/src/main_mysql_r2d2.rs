use serde::Serialize;
use spring::{auto_config, App};

use spring_diesel_orm::diesel_sync::{DieselOrmPlugin, MysqlR2d2ConnectionPool};
use spring_web::get;
use spring_web::{
    axum::response::{IntoResponse, Json},
    error::Result,
    extractor::Component,
    WebConfigurator, WebPlugin,
};

use anyhow::Context;

use diesel::prelude::*;
use crate::schema::users;



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
        .add_plugin(DieselOrmPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await;
}

#[get("/users")]
async fn get_users(Component(db): Component<MysqlR2d2ConnectionPool>) -> Result<impl IntoResponse> {
    let mut connection = db.get().context("failed to get db connection")?;    
    let rows: Vec<User> = users::table
        .filter(users::active.eq(true))
        .limit(10)
        .load(&mut connection)
        .context("query users failed")?;
    //.await.context("query users failed")?;
    Ok(Json(rows))
}
