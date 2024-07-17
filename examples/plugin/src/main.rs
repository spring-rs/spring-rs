use async_trait::async_trait;
use autumn_boot::{app::App, plugin::Plugin};
use serde::Deserialize;

struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    async fn build(&self, app: &mut autumn_boot::app::App) {
        match app.get_config::<Config>(self) {
            Ok(config) => println!("{:#?}", config),
            Err(e) => println!("{}", e),
        }
    }

    fn config_prefix(&self) -> &str {
        "my-plugin"
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    a: u32,
    b: bool,
    c: ConfigInner,
    d: String,
    e: ConfigEnum,
}

#[derive(Debug, Deserialize)]
enum ConfigEnum {
    EA,
    EB,
    EC,
    ED,
}

#[derive(Debug, Deserialize)]
struct ConfigInner {
    f: u32,
    g: String,
}

#[tokio::main]
pub async fn main() {
    App::new().add_plugin(MyPlugin).run().await;
    println!("finish");
}
