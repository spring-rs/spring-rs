## Introduction

The log plugin is a plugin built into the `spring` core module and is also the first plugin loaded by spring. This plugin does not need to be added manually by the developer.

This plugin integrates the most popular log library [`tracing`](https://tracing.rs/) and provides the following configuration items

## Configuration items

```toml
[logger]
enable = true            # Whether to enable the log function, which is enabled by default
pretty_backtrace = false # Whether to print stack information, which is disabled by default. It is recommended to enable it only during application development
level = "info"           # The default log level is info
format = "compact"       # The log format supports compact, pretty, and json, and the default is compact
override_filter = "info,axum=debug" # Override the default log filter level, and specify the log level for the crate library
file = { enabled = true } # Whether to write logs to files, which is not enabled by default
```

You can also configure the log file in more detail
```toml
[logger.file]
enabled = true      # Whether to write logs to files
non_blocking = true # Whether to enable non-blocking writing, which is enabled by default
format = "compact"  # Log format supports compact, pretty, json, the default is compact
rotation = "daily"  # Log rotation mode minutely, hourly, daily, never, the default is daily rotation
dir = "./logs"      # Log file directory
filename_prefix = "app" # Log file prefix name
filename_suffix = "log" # Log file suffix name
max_log_files = 365     # Maximum number of logs to retain
```