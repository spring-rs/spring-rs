<b>spring-rs</b> is a microservice framework written in rust, similar to SpringBoot in java. <b>spring-rs</b> provides an easily extensible plug-in system for integrating excellent projects in the rust community, such as axum, sqlx, sea-orm, etc.

## Example

```rust
use spring::{get, auto_config, App};
use spring_web::{response::IntoResponse, WebConfigurator, WebPlugin};

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .run()
        .await
}

#[get("/")]
async fn hello_world() -> impl IntoResponse {
    "hello world"
}
```

## Supported plugins

* [x] [`spring-web`](https://docs.rs/spring-web)(Based on [`axum`](https://github.com/tokio-rs/axum))
* [x] [`spring-sqlx`](https://docs.rs/spring-sqlx)(Integrated with [`sqlx`](https://github.com/launchbadge/sqlx))
* [x] [`spring-sea-orm`](https://docs.rs/spring-sea-orm)(Integrated with [`sea-orm`](https://www.sea-ql.org/SeaORM/))
* [x] [`spring-redis`](https://docs.rs/spring-redis)(Integrated with [`redis`](https://github.com/redis-rs/redis-rs))
* [x] [`spring-mail`](https://docs.rs/spring-mail)(integrated with [`lettre`](https://github.com/lettre/lettre))
* [x] [`spring-job`](https://docs.rs/spring-job)(integrated with [`tokio-cron-scheduler`](https://github.com/mvniekerk/tokio-cron-scheduler))
* [ ] `spring-actuator`(provides a simple health check and system diagnostic interface)
* [ ] `spring-stream`(integrated with [`sea-streamer`](https://github.com/SeaQL/sea-streamer) to implement message processing)
* [ ] `spring-opentelemetry`(integrated with [`opentelemetry`](https://github.com/open-telemetry/opentelemetry-rust) to implement full observability of logging, metrics, tracing)
