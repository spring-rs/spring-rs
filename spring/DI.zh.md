## 编译期依赖注入

spring-rs提供了一种特殊的Component——[Service](https://docs.rs/spring/latest/spring/plugin/service/index.html)，它支持在编译期注入依赖的组件。

像下面的例子`UserService`只需派生`Service`特征，为了区分注入的依赖，你需要通过属性宏`#[inject(component)]`和`#[inject(config)]`指定依赖是一个Component还是一个Config。

```rust
use spring_sqlx::ConnectPool;
use spring::config::Configurable;
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

完整代码参考[`dependency-inject-example`](https://github.com/spring-rs/spring-rs/tree/master/examples/dependency-inject-example)。

> Service还支持grpc模式，可结合[spring-grpc](https://spring-rs.github.io/zh/docs/plugins/spring-grpc/)插件一起使用