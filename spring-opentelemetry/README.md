[![crates.io](https://img.shields.io/crates/v/spring-opentelemetry.svg)](https://crates.io/crates/spring-opentelemetry)
[![Documentation](https://docs.rs/spring-opentelemetry/badge.svg)](https://docs.rs/spring-opentelemetry)

## Dependencies

```toml
spring-opentelemetry = "<version>"
```

OTEL uses the [W3C format](https://github.com/w3c/trace-context) to pass context information for link tracing by default.

Optional features:
* `jaeger`: Use [jaeger format](https://www.jaegertracing.io/docs/1.18/client-libraries/#propagation-format) to propagate context
* `zipkin`: Use [zipkin format](https://github.com/openzipkin/b3-propagation) to propagate context
* `more-resource`: Add more resource information, such as host Host, operating system, process information

For complete code, refer to [`opentelemetry-example`](https://github.com/spring-rs/spring-rs/tree/master/examples/opentelemetry-example)