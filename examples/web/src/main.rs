use autumn_boot::app::App;
use autumn_boot_sqlx::SqlxPlugin;
use autumn_boot_web::WebPlugin;

#[tokio::main]
async fn main() {
    App::new()
        .config_file("/Users/holmofy/rust/autumn-boot/examples/web/config/app.toml")
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await
}
