use std::time::Duration;

use spring::App;
use spring_web::get;
use spring_web::{axum::response::IntoResponse, Router, WebConfigurator, WebPlugin};
use tower_http::timeout::TimeoutLayer;

#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .add_router(router())
        .run()
        .await
}

fn router() -> Router {
    Router::new()
        .merge(spring_web::handler::auto_router())
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
}

#[get("/")]
async fn hello_world() -> impl IntoResponse {
    "hello world"
}
