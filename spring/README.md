[![crates.io](https://img.shields.io/crates/v/spring.svg)](https://crates.io/crates/spring)
[![Documentation](https://docs.rs/spring/badge.svg)](https://docs.rs/spring)

## Introduction

`spring` is the core module of the `spring` project, which includes: configuration management, plugin management, and component management.

All plugins need to implement the [`Plugin`](https://docs.rs/spring/latest/spring/plugin/trait.Plugin.html) feature.

## How to write your own plugin

Add dependencies

```toml
spring = { version = "0.0.9" }           # This crate contains the definition of plugin traits
serde = { workspace = true, features = ["derive"] } # Used to parse plugin configuration items
```

```rust
use serde::Deserialize;
use spring::async_trait;
use spring::config::Configurable;
use spring::{app::AppBuilder, plugin::Plugin};

struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        // Call app.get_config::<Config>() method to get configuration items
        match app.get_config::<Config>() {
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

/// Plugin configuration
#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "my-plugin"]
struct Config {
    a: u32,
    b: bool,
}
```

For the complete code, refer to [`plugin-example`](https://github.com/spring-rs/spring-rs/tree/master/examples/plugin-example), or refer to other built-in plugin codes.