+++
title = "Auto-Reloading Development Server"
description = "During development it can be very handy to have cargo automatically recompile the code on changes. "
draft = false
weight = 101
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = "During development it can be very handy to have cargo automatically recompile the code on changes."
toc = true
top = false
+++

This can be accomplished very easily by using [cargo-watch](https://github.com/watchexec/cargo-watch).

```sh
cargo watch -x run
```

# Using hot reloading

If you want to go a step further and have your application automatically reload code without restarting the server, you can use
the feature of hot reloading. This is especially useful when you want to keep the application state (e.g. logged in user, form data, etc.) while making changes to the code.

To enable hot reloading, you need to enable the `hot-reload` feature in your `Cargo.toml`:

```toml
[features]
default = []
hot-reload = ["spring/hot-reload"]
```

And add the following dependencies:

```toml
subsecond = { git = "https://github.com/DioxusLabs/dioxus.git"}
dioxus-devtools = { git = "https://github.com/DioxusLabs/dioxus.git", features = ["serve"]}
```

This is because the hot reloading feature relies on `subsecond` and `dioxus-devtools` to function properly.

And they don't come by default with the `spring` crate, besides we are using a custom git version because the crates are not published to crates.io yet.

In our `main.rs` file, we need to change the code to enable hot reloading when the feature is enabled, this behavior can be controlled using our macro `spring::main`, it's just a re export of the `tokio::main` macro with this extra functionality.

```diff
- #[tokio::main]
+ #[spring::main]
fn main() {
    // ...
}
```

Now we must to install the external `dioxus-cli` tool, this will allow us to run the development server with hot reloading support.

```sh
cargo install dioxus-cli@0.7.0-alpha.3
```

Finally, we can run our application with hot reloading support using the `dx serve` command:

```sh
dx serve --hot-patch --features hot-reload
```

This will start the development server and automatically reload the code when changes are detected, without restarting the server.

> **Note**: Hot reloading is still an experimental feature.


