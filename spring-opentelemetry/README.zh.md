[![crates.io](https://img.shields.io/crates/v/spring-opentelemetry.svg)](https://crates.io/crates/spring-opentelemetry)
[![Documentation](https://docs.rs/spring-opentelemetry/badge.svg)](https://docs.rs/spring-opentelemetry)

## 依赖

```toml
spring-opentelemetry = "<version>"
```

OTEL默认使用[W3C格式](https://github.com/w3c/trace-context)传递链路追踪的上下文信息。

可选的features: 
* `jaeger`: 使用[jaeger格式](https://www.jaegertracing.io/docs/1.18/client-libraries/#propagation-format)透传上下文
* `zipkin`: 使用[zipkin格式](https://github.com/openzipkin/b3-propagation)透传上下文
* `more-resource`: 添加更多的资源信息，如主机Host、操作系统、进程信息

完整代码参考[`opentelemetry-example`](https://github.com/spring-rs/spring-rs/tree/master/examples/opentelemetry-example)