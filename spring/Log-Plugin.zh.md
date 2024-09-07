## 介绍

日志插件是内置在`spring`核心模块的插件，也是spring加载的第一个插件。这个插件不需要开发者手动添加。

该插件是集成了最流行的日志库[`tracing`](https://tracing.rs/)，并提供了以下配置项

## 配置项

```toml
[logger]
enable = true                               # 是否启用日志功能，默认是开启的
pretty_backtrace = false                    # 是否打印堆栈信息，默认是关闭的，建议只在应用开发阶段开启
level = "info"                              # 默认日志级别是info
format = "compact"                          # 日志格式支持compact、pretty、json，默认是compact
override_filter = "info,axum=debug"         # 重写默认的日志过滤级别，可以针对crate库指定日志级别
file = { enabled = true }                   # 是否将日志写入文件中，默认没有开启
```

你也可以对日志文件进行更详细的配置
```toml
[logger.file]
enabled = true                              # 是否将日志写入文件
non_blocking = true                         # 是否启用非阻塞方式写入，默认开启
format = "compact"                          # 日志格式支持compact、pretty、json，默认是compact
rotation = "daily"                          # 日志滚动方式minutely、hourly、daily、never，默认按天滚动
dir = "./logs"                              # 日志文件目录
filename_prefix = "app"                     # 日志文件前缀名
filename_suffix = "log"                     # 日志文件后缀名
max_log_files = 365                         # 保留的最大日志数量
```