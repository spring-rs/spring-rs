# Changelog

## 0.2.5

- **changed**: fix exported `build` method ([#75])

[#75]: https://github.com/spring-rs/spring-rs/pull/75

## 0.2.4

- **added**: App add `get_env` ([#74])
- **added**: AppBuilder export `build` method ([#74])

[#74]: https://github.com/spring-rs/spring-rs/pull/74

## 0.2.3

- **added**: AppBuilder add `get_env` ([#65])
- **added**: AppBuilder add `add_layer` to support extends `tracing-rs` ([#65])
- **added**: support immediately Plugin ([#65])

[#65]: https://github.com/spring-rs/spring-rs/pull/65

## 0.2.2

- **added**: support shutdown hook ([#61])

[#61]: https://github.com/spring-rs/spring-rs/pull/61

## 0.2.1

- **added**: support config logger time pattern ([#59])
- **added**: support config logger add with_fields ([#59])

[#59]: https://github.com/spring-rs/spring-rs/pull/59

## 0.2.0

- **added**: add `Service` support dependency inject ([#54])
- **added**: add `get_component` ([#54])
- **breaking**: rename origin `get_component` to `get_component_ref` ([#54])

[#54]: https://github.com/spring-rs/spring-rs/pull/54

## 0.1.3

- **changed**: refactor toml config registry ([#1a750a])

[#1a750a]: https://github.com/spring-rs/spring-rs/commit/1a750a7d82871632bad7cee73ec418b5a28924ea

## 0.1.2

- **changed**: add DeserializeErr ([#44])

[#44]: https://github.com/spring-rs/spring-rs/pull/44

## before 0.1.1

see [CHANGELOG](../CHANGELOG.md)