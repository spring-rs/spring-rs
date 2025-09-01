+++
title = "自动热重启"
description = "在代码变更后自动重启服务"
draft = false
weight = 101
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = "代码变更后自动编译并重启服务，这在开发过程中会非常方便。"
toc = true
top = false
+++

使用[`cargo-watch`](https://github.com/watchexec/cargo-watch)可以轻松实现这个功能。

```sh
cargo watch -x run
```

# 使用热重载

如果你想更进一步，让应用程序在不重启服务器的情况下自动重新加载代码，你可以使用 **热重载（hot reloading）** 功能。
这在你希望在修改代码的同时保持应用状态（例如：已登录用户、表单数据等）时尤其有用。

要启用热重载，你需要在 `Cargo.toml` 中启用 `hot-reload` 功能：

```toml
[features]
default = []
hot-reload = ["spring/hot-reload"]
```

并添加以下依赖：

```toml
subsecond = { git = "https://github.com/DioxusLabs/dioxus.git"}
dioxus-devtools = { git = "https://github.com/DioxusLabs/dioxus.git", features = ["serve"]}
```

这是因为热重载功能依赖 `subsecond` 和 `dioxus-devtools` 才能正常运行。

它们并不是 `spring` crate 默认自带的，而且我们使用的是自定义的 git 版本，因为这些 crate 尚未发布到 crates.io。

在我们的 `main.rs` 文件中，需要修改代码，以便在启用该功能时支持热重载。
这个行为可以通过 `spring::main` 宏来控制，它只是对 `tokio::main` 宏的一个再导出，并附加了热重载功能：

```diff
- #[tokio::main]
+ #[spring::main]
fn main() {
    // ...
}
```

接下来，我们需要安装外部工具 `dioxus-cli`，这样就能运行带有热重载支持的开发服务器：

```sh
cargo install dioxus-cli@0.7.0-alpha.3
```

最后，可以使用以下命令运行带有热重载支持的应用：

```sh
dx serve --hot-patch --features hot-reload
```

这将启动开发服务器，并在检测到代码更改时自动重新加载，而无需重启服务器。

> **注意**：热重载功能仍然是实验性的。
