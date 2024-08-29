+++
title = "Introduction"
description = "spring-rs is a application framework written in Rust, similar to SpringBoot in java ecosystem. spring-rs provides an easily extensible plug-in system for integrating excellent projects in Rust community, such as axum, sqlx, sea-orm, etc."
draft = false
weight = 2
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = 'spring-rs is a application framework written in Rust, similar to SpringBoot in java ecosystem. spring-rs provides an easily extensible plug-in system for integrating excellent projects in Rust community, such as axum, sqlx, sea-orm, etc.'
toc = true
top = false
+++

## Quick Start

The premise of using spring-rs is that you are familiar with the basic syntax of Rust and the usage of cargo dependency package management tool.

If you already know these prerequisites, click this [Quick Start →](/docs/getting-started/quick-start/), which introduces how to quickly get started with **spring-rs**.

## Supported plugins

* [x] [`spring-web`](/docs/plugins/spring-web/)(Based on [`axum`](https://github.com/tokio-rs/axum))
* [x] [`spring-sqlx`](/docs/plugins/spring-sqlx/)(Integrated with [`sqlx`](https://github.com/launchbadge/sqlx))
* [x] [`spring-sea-orm`](/docs/plugins/spring-sea-orm/)(Integrated with [`sea-orm`](https://www.sea-ql.org/SeaORM/))
* [x] [`spring-redis`](/docs/plugins/spring-redis/)(Integrated with [`redis`](https://github.com/redis-rs/redis-rs))
* [x] [`spring-mail`](/docs/plugins/spring-mail/)(integrated with [`lettre`](https://github.com/lettre/lettre))
* [x] [`spring-job`](/docs/plugins/spring-job/)(integrated with [`tokio-cron-scheduler`](https://github.com/mvniekerk/tokio-cron-scheduler))
* [x] [`spring-stream`](/docs/plugins/spring-stream/)(Integrate [`sea-streamer`](https://github.com/SeaQL/sea-streamer) to implement message processing such as redis-stream and kafka)
* [ ] `spring-actuator`(provides a simple health check and system diagnostic interface)
* [ ] `spring-opentelemetry`(integrate with [`opentelemetry`](https://github.com/open-telemetry/opentelemetry-rust) to implement full observability of logging, metrics, tracing)
* [ ] `spring-tarpc`(Integrate[`tarpc`](https://github.com/google/tarpc) to implement RPC calls)

## Contribution

We also welcome community experts to contribute their own plugins. [Contributing →](https://github.com/spring-rs/spring-rs)

## Help

Click here to view common problems encountered when using `spring-rs` [Help →](../../help/faq/)