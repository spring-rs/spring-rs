+++
title = "rust的spring-rs和java的springboot数据对比"
description = "rust的spring-rs和java的springboot相关压测报告"
sort_by = "weight"
date = 2024-09-04T09:19:42+00:00
updated = 2024-09-04T09:19:42+00:00
weight = 3
draft = false
template = "blog/page.html"
+++

使用spring-rs构建的release版二进制文件大小是SpringBoot jar包的一半。
<img width="100%" alt="Build Target Size" src="https://quickchart.io/chart?c={type:%27bar%27,data:{labels:[%22java%27sspring-boot%22,%22rust%27sspring-rs%22],datasets:[{label:%22build%20target%20size(MB)%22,data:[22.25,11.17]}]}}&format=svg"/>

使用spring-rs构建的Docker镜像大小是SpringBoot镜像的1/4。
<img width="100%" alt="Docker Image Size" src="https://quickchart.io/chart?c={type:%27bar%27,data:{labels:[%22java-springboot%22,%22rust-spring-rs%22],datasets:[{label:%27docker%20image%20size(MB)%27,data:[429.99,124.55]}]}}&format=svg"/>

使用spring-rs的运行时内存占用是SpringBoot运行时占用的1/10。
<img width="100%" alt="Runtime Memory Usage" src="https://quickchart.io/chart?c={type:%27bar%27,data:{labels:[%22java-springboot%22,%22rust-spring-rs%22],datasets:[{label:%27Runtime%20Memory%20Usage(MB)%27,data:[234.6,21.2]}]}}&format=svg"/>

使用spring-rs的最简单的Web应用程序QPS是SpringBoot的5倍。
<img width="100%" alt="Raw Query QPS" src="https://quickchart.io/chart?c={type:%27bar%27,data:{labels:[%22java-springboot%22,%22rust-spring-rs%22],datasets:[{label:%27Raw%20Query%20QPS%27,data:[24805.60,40143.45]}]}}&format=svg"/>

使用spring-rs的包含数据库查询的Web应用程序QPS和SpringBoot基本相当。
<img width="100%" alt="Postgres Query QPS" src="https://quickchart.io/chart?c={type:%27bar%27,data:{labels:[%22java-springboot%22,%22rust-spring-rs%22],datasets:[{label:%27Postgres%20Query%20QPS%27,data:[9679.59,9250.40]}]}}&format=svg"/>

目前用的数据查询工具是`sqlx`，[sqlx对mysql性能](https://github.com/launchbadge/sqlx/issues/1481)很差，压测结果只有SpringBoot的一半，所以推荐使用PostgreSQL作为sqlx的后端。

接下来我会对接一下[rust-postgres](https://github.com/sfackler/rust-postgres)，看看性能会不会比sqlx有所提升。

详细压测代码和相关数据可以[点击这个链接](https://github.com/spring-rs/spring-benchmark)