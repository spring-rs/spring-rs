# Changelog

## Unrelease

- **added:** [#14 spring-tarpc plugin](https://github.com/spring-rs/spring-rs/issues/14)

## after 0.1.2 CHANGELOG

* [spring CHANGELOG](./spring/CHANGELOG.md)
* [spring-job CHANGELOG](./spring-job/CHANGELOG.md)
* [spring-macros CHANGELOG](./spring-macros/CHANGELOG.md)
* [spring-mail CHANGELOG](./spring-mail/CHANGELOG.md)
* [spring-postgres CHANGELOG](./spring-postgres/CHANGELOG.md)
* [spring-redis CHANGELOG](./spring-redis/CHANGELOG.md)
* [spring-sea-orm CHANGELOG](./spring-sea-orm/CHANGELOG.md)
* [spring-sqlx CHANGELOG](./spring-sqlx/CHANGELOG.md)
* [spring-stream CHANGELOG](./spring-stream/CHANGELOG.md)
* [spring-web CHANGELOG](./spring-web/CHANGELOG.md)

## 0.1.1 - 2024.9.8

- **added**: spring-sea-orm add PaginationExt trait. ([#commit_003715])

[#commit_003715]: https://github.com/spring-rs/spring-rs/commit/003715f843c0200d6e46db206f03eed135ff9ddb

## 0.1.0 - 2024.9.8

- **added**: add ConfigRegistry trait. ([#31])
- **added**: add Config extractor for spring-web,spring-job,spring-stream. ([#31])
- **breaking**: refactor app configuration management: Configuration and plugins are independent of each other. ([#31])

[#31]: https://github.com/spring-rs/spring-rs/pull/31

**Migrating from 0.0 to 0.1**

```diff
-#[derive(Configurable)]
-#[config_prefix = "my-plugin"]
struct MyPlugin;
```

```diff
 #[derive(Debug, Configurable, Deserialize)]
+#[config_prefix = "my-plugin"]
 struct Config {
     a: u32,
     b: bool,
 }
```

## 0.0.9 - 2024.9.4

- **added**: spring-postgres plugin
- **added**: spring-boot testcase
- **changed**: fix spring-web default binding ip
- **changed**: the added component must implement the Clone trait
- **removed**: spring-actuator

## 0.0.8 - 2024.8.25

- **added:** [#3 spring-stream plugin](https://github.com/spring-rs/spring-rs/issues/3) ([#21])

[#21]: https://github.com/spring-rs/spring-rs/pull/21

## 0.0.7 - 2024.8.21

- **added:** spring-web add KnownWebError ([#19])
- **added:** [#18 jwt login example](https://github.com/spring-rs/spring-rs/issues/18)

[#19]: https://github.com/spring-rs/spring-rs/pull/19

## 0.0.0 - 2024.7.15

Initial implementation of spring-boot plugin system

- **added:** [Plugin System](https://github.com/holmofy/spring-boot/pull/2)
