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
