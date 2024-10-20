你可以通过下面的方式定义配置：
```rust
#[derive(Debug, Configurable, Deserialize)]
#[config_prefix = "my-plugin"]
struct Config {
    a: u32,
    b: bool,
}
```

通过[`app.get_config()`][ConfigRegistry::get_config()]方法可以读取`toml`中的配置：

```toml
[my-plugin]
a = 10
b = true
```