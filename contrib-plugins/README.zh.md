* [opendal](#spring-opendal): OpenDAL 提供了统一的数据访问层, 可以方便的访问各种存储系统.

## Spring OpenDAL

[spring-opendal](spring-opendal) 集成 [Apache OpenDAL™](https://opendal.apache.org/) 到 spring-rs 中,
可以为所有类型的存储系统提供了本地支持, 包括对象存储服务, 文件存储服务, 以及许多

具体的例子可以参考 [with-spring-web](spring-opendal/examples/with-spring-web) 项目.

- 运行实例代码

```shell
cargo run --color=always --package spring-opendal --example with-spring-web --features=services-fs
```

- 运行 blocking 测试代码

```shell
cargo test --test blocking --features="services-memory layers-blocking test-layers" -- --nocapture
```
