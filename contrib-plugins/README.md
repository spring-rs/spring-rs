* [opendal](#spring-opendal): OpenDAL offers a unified data access layer, empowering users to
  seamlessly and efficiently retrieve data from diverse storage services.

## Spring OpenDAL

[spring-opendal](spring-opendal) integrates [Apache OpenDAL™](https://opendal.apache.org/) into
spring-rs, providing native support for all types of storage systems, including object storage
services, file storage services, and many more.

For specific examples, please refer to
the [with-spring-web](https://github.com/spring-rs/spring-rs/tree/master/contrib-plugins/spring-opendal/examples/with-spring-web) project.

- Run the example

```shell
cargo run --color=always --package spring-opendal --example with-spring-web --features=services-fs
```

- Run the blocking test

```shell
cargo test --test blocking --features="services-memory layers-blocking test-layers" -- --nocapture
```
