You can define configuration in the following way:
```rust
#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "my-plugin"]
struct Config {
    a: u32,
    b: bool,
}
```

The configuration in `toml` can be read through the [`app.get_config()`](https://docs.rs/spring/latest/spring/app/struct.AppBuilder.html#method.get_config) method:

```toml
[my-plugin]
a = 10
b = true
```

```rust
struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        // Loading configuration in your own plugin
        let config = app.get_config::<Config>().expect("load config failed");
        // do something...
    }
}
```

## Use configuration in other plugins

* [`spring-web`](https://spring-rs.github.io/docs/plugins/spring-web/#read-configuration)
* [`spring-job`](https://spring-rs.github.io/docs/plugins/spring-job/#read-configuration)
* [`spring-stream`](https://spring-rs.github.io/docs/plugins/spring-stream/#read-configuration)