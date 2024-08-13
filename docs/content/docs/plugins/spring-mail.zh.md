+++
title = "spring-mail插件"
description = "mail插件如何使用"
draft = false
weight = 30
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = "spring-mail是基于<a href='https://github.com/lettre/lettre' target='_blank'>lettre</a>实现"
toc = true
top = false
+++


![lettre Repo stars](https://img.shields.io/github/stars/lettre/lettre) ![downloads](https://img.shields.io/crates/d/lettre.svg)
lettre是rust最流行的邮件客户端，并且支持异步API。spring-mail主要使用它的tokio异步API。


{{ include(path="../../spring-mail/README.zh.md") }}