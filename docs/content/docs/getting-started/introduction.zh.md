+++
title = "介绍"
description = "spring-rs是一个Rust编写的应用框架，类似于java生态的springboot"
draft = false
weight = 2
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = '<b>spring-rs</b>是一个Rust编写的应用框架，类似于java生态的springboot。<b>spring-rs</b>提供了易于扩展的插件系统，用于整合Rust社区的优秀项目，例如axum、sqlx、sea-orm等。'
toc = true
top = false
+++

## 快速上手

使用spring-rs的前提是，您已熟悉Rust基本语法和Cargo依赖包管理工具的使用。

如果这些你都已了解，点击这个[Quick Start →](/zh/docs/getting-started/quick-start/)，它介绍了如何快速上手**spring-rs**。

## 支持的插件

* [x] [`spring-web`](/zh/docs/plugins/spring-web/)(基于[`axum`](https://github.com/tokio-rs/axum)实现)
* [x] [`spring-sqlx`](/zh/docs/plugins/spring-sqlx/)(整合了[`sqlx`](https://github.com/launchbadge/sqlx))
* [x] [`spring-postgres`](/zh/docs/plugins/spring-postgres/)(整合了[`rust-postgres`](https://github.com/sfackler/rust-postgres))
* [x] [`spring-sea-orm`](/zh/docs/plugins/spring-sea-orm/)(整合了[`sea-orm`](https://www.sea-ql.org/SeaORM/))
* [x] [`spring-redis`](/zh/docs/plugins/spring-redis/)(整合了[`redis`](https://github.com/redis-rs/redis-rs))
* [x] [`spring-mail`](/zh/docs/plugins/spring-mail/)(整合了[`lettre`](https://github.com/lettre/lettre))
* [x] [`spring-job`](/zh/docs/plugins/spring-job/)(整合了[`tokio-cron-scheduler`](https://github.com/mvniekerk/tokio-cron-scheduler))
* [x] [`spring-stream`](/zh/docs/plugins/spring-stream/)(整合了[`sea-streamer`](https://github.com/SeaQL/sea-streamer)实现redis-stream、kafka等消息处理)
* [ ] `spring-opentelemetry`(整合了[`opentelemetry`](https://github.com/open-telemetry/opentelemetry-rust)实现logging、metrics、tracing全套可观测性)
* [ ] `spring-tarpc`(整合了[`tarpc`](https://github.com/google/tarpc)实现RPC调用)

## 贡献

也欢迎社区的大牛贡献自己的插件。 [Contributing →](https://github.com/spring-rs/spring-rs)

## 帮助

点击这里可以查看`spring-rs`使用过程中遇到的常见问题 [Help →](../../help/faq/)