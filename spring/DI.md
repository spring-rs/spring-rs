## Compile-time dependency inject

spring-rs provides a special Component - [Service](https://docs.rs/spring/latest/spring/plugin/service/index.html), which supports injecting dependent components at compile time.

Like the following example, `UserService` only needs to derive the `Service` trait. In order to distinguish the injected dependencies, you need to specify whether the dependency is a Component or a Config through the attribute macros `#[inject(component)]` and `#[inject(config)]`.

```rust
use spring_sqlx::ConnectPool;
use spring::config::Configurable;
use spring::plugin::service::Service;
use serde::Deserialize;

#[derive(Clone, Configurable, Deserialize)]
#[config_prefix = "user"]
struct UserConfig {
    username: String,
    project: String,
}

#[derive(Clone, Service)]
struct UserService {
    #[inject(component)]
    db: ConnectPool,
    #[inject(config)]
    config: UserConfig,
}
```

For the complete code, see [`dependency-inject-example`](https://github.com/spring-rs/spring-rs/tree/master/examples/dependency-inject-example).