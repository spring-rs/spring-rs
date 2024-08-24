[![crates.io](https://img.shields.io/crates/v/spring-stream.svg)](https://crates.io/crates/spring-stream)
[![Documentation](https://docs.rs/spring-stream/badge.svg)](https://docs.rs/spring-stream)

## 依赖

```toml
spring-stream = { version = "0.0.7" }
```

## 配置项

```toml
[stream]
uri = "file://./stream"      # StreamerUri 数据流地址
```

StreamUri支持file、stdio、redis、kafka。uri的格式具体参考[StreamerUri](https://docs.rs/sea-streamer/latest/sea_streamer/struct.StreamerUri.html)。

* stdio适合命令行项目。
* file适合单机部署的项目。
* redis适合分布式部署的项目。Redis5.0提供了stream数据结构，因此要求redis版本大于5.0。详细可以参考[redis stream官方文档](https://redis.io/docs/latest/develop/data-types/streams/)。
* kafka适合消息量更大的分布式部署的项目。Kafka可以用兼容[redpanda](https://github.com/redpanda-data/redpanda)替代，它是C++编写的兼容kafka协议的高性能消息中间件，用它可以彻底摆脱Kafka依赖的JVM。

### 流的详细配置
```toml
# 文件流配置
[stream.file]
connect = { create_file = "CreateIfNotExists" }

# 标准流配置
[stream.stdio]
connect = { loopback = false }

# redis流配置
[stream.redis]
connect = { db=0,username="user",password="passwd" }

# kafka流配置
[stream.kafka]
connect = { sasl_options={mechanism="Plain",username="user",password="passwd"}}
```
