[![crates.io](https://img.shields.io/crates/v/spring-boot.svg)](https://crates.io/crates/spring-boot)
[![Documentation](https://docs.rs/spring-boot/badge.svg)](https://docs.rs/spring-boot)

## Introduction

`spring-boot` is the core module of the `spring` project, which includes: configuration management, plugin management, and component management.

All plugins need to implement the [`Plugin`](https://docs.rs/spring-boot/latest/spring_boot/plugin/trait.Plugin.html) feature.

## How to write your own plugin

Add dependencies

```toml
spring-boot = { version = "0.0.8" }      # This crate contains the definition of plugin traits
serde = { workspace = true, features = ["derive"] } # Used to parse plugin configuration items
```

```rust
use serde::Deserialize;
use spring_boot::async_trait;
use spring_boot::config::Configurable;
use spring_boot::{app::AppBuilder, plugin::Plugin};

struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        // Call app.get_config::<Config>(self) method to get configuration items
        match app.get_config::<Config>(self) {
            Ok(config) => {
                println!("{:#?}", config);
                assert_eq!(config.a, 1);
                assert_eq!(config.b, true);

                // Get the configuration items to build the corresponding components

            }
            Err(e) => println!("{:?}", e),
        }
    }
}

/// Configuration item prefix
impl Configurable for MyPlugin {
    fn config_prefix(&self) -> &str {
        "my-plugin"
    }
}

/// Plugin configuration
#[derive(Debug, Deserialize)]
struct Config {
    a: u32,
    b: bool,
}
```

You can use the derive macro to implement the Configurable trait:

```rust
/// Use the `config_prefix` attr macro to define the prefix configured in the toml file
#[derive(Configurable)]
#[config_prefix = "my-plugin"]
struct MyPlugin;
```

For the complete code, refer to [`plugin-example`](https://github.com/spring-rs/spring-rs/tree/master/examples/plugin-example), or refer to other built-in plugin codes.