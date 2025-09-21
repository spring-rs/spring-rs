use spring::{auto_config, App};
use spring_sqlx::{SqlxPlugin};
use spring_web::{WebConfigurator, WebPlugin};
use spring_web::{
    axum::response::IntoResponse,
    get,
};

#[auto_config(WebConfigurator)]
#[spring::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .add_plugin(SqlxPlugin)
        .run()
        .await;
}

#[get("/")]
async fn hello_world() -> impl IntoResponse {
    "Remove this line and uncomment the code below to test hot reloading!"
    // "🎉 **CONFIRMED WORKING** - Hot reload is functioning perfectly! 🚀🔥⚡✨"
}
