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