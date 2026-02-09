# Changelog

## 0.4.2

- **added**: `lazy_storage<T>()` function for custom storage backends using `#[derive(Service)]`
- **added**: `prelude.rs` module for simplified imports (all types re-exported from `spring_sa_token`)
- **added**: `custom_storage.rs` module for custom storage support
- **changed**: `sa_token_auth()` renamed to `sa_token_configure()`
- **changed**: `configure()` renamed to `configure_path_auth()`

## 0.4.1

- **added**: Initial release of spring-sa-token plugin