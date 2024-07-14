use autumn_boot::{app::App, error::Result, plugin::Plugin};
use serde::Deserialize;

struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut autumn_boot::app::App) {
        todo!()
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

pub fn main() -> Result<()> {
    let mut app = App::new();
    match app.run() {
        Err(e) => println!("{:?}", e),
        _ => println!("running"),
    }
    let plugin = MyPlugin;
    match app.get_config::<Config>(plugin) {
        Ok(config) => println!("{:#?}", config),
        Err(e) => println!("{}", e),
    }
    println!("finish");

    Ok(())
}
