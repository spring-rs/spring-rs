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

