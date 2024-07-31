mod entities;

use anyhow::Context;
use autumn_boot::app::App;
use autumn_sea_orm::{DbConn, SeaOrmPlugin};
use autumn_web::{
    error::Result,
    extractor::{Component, Path, Query},
    get,
    response::{IntoResponse, Json},
    Router, WebConfigurator, WebPlugin,
};
use entities::{
    prelude::{TodoItem, TodoList},
    todo_item, todo_list,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, QueryTrait};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(SeaOrmPlugin)
        .add_plugin(WebPlugin)
        .add_router(router())
        .run()
        .await
}

fn router() -> Router {
    Router::new()
        .route("/", get(todo_list))
        .route("/:id", get(get_todo_list))
}

#[derive(Deserialize)]
struct TodoListQuery {
    title: Option<String>,
    page: Option<u64>,
    size: Option<u64>,
}

impl TodoListQuery {
    fn offset(&self) -> Option<u64> {
        Some(self.page? * self.size?)
    }
}

async fn todo_list(
    Component(db): Component<DbConn>,
    Query(query): Query<TodoListQuery>,
) -> Result<impl IntoResponse> {
    let offset = query.offset();
    let rows = TodoList::find()
        .apply_if(query.title, |query, v| {
            query.filter(todo_list::Column::Title.starts_with(v))
        })
        .apply_if(query.size, QuerySelect::limit)
        .apply_if(offset, QuerySelect::offset)
        .all(&db)
        .await
        .context("query todo list failed")?;
    Ok(Json(rows))
}

async fn get_todo_list(
    Component(db): Component<DbConn>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let rows = TodoItem::find()
        .filter(todo_item::Column::ListId.eq(id))
        .all(&db)
        .await
        .context("query todo list failed")?;
    Ok(Json(rows))
}
