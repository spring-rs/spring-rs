+++
title = "spring-mail Plugin"
description = "How to use the spring-mail plugin"
draft = false
weight = 16
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = "spring-mail is an automatic assembly for <a href='https://github.com/lettre/lettre' target='_blank'>lettre</a>"
toc = true
top = false
+++

![lettre Repo stars](https://img.shields.io/github/stars/lettre/lettre) ![downloads](https://img.shields.io/crates/d/lettre.svg)
Lettre is the most popular mail client in Rust and supports asynchronous API. spring-mail mainly uses its tokio asynchronous API.

{{ include(path="../../spring-mail/README.md") }}