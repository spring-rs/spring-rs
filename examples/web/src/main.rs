use autumn_boot::app::App;
use autumn_boot_sqlx::SqlxPlugin;
use autumn_boot_web::{get, Router, WebConfigurator, WebPlugin};

#[tokio::main]
async fn main() {
    App::new()
        .config_file("/Users/holmofy/rust/autumn-boot/examples/web/config/app.toml")
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        .add_router(router())
        .run()
        .await
}

fn router() -> Router {
    Router::new().route("/", get(hello_word))
}

async fn hello_word() -> &'static str {
    "hello word"
}
