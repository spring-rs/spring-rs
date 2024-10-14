+++
title = "opentelemetry插件"
description = "opentelemetry插件如何使用"
draft = false
weight = 22
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = "OpenTelemetry是一个集log、metrics、tracing于一体的全维度可观测性方案"
toc = true
top = false
+++

## OpenTelemetry简史

* 2010年 Google发布的 [Dapper](https://research.google/pubs/dapper-a-large-scale-distributed-systems-tracing-infrastructure/) 论文是分布式链路追踪的开端
* 2012年 Twitter 开源了 [Zipkin](https://zipkin.io/)。
* 2015年 Uber 发布了 [Jaeger](https://www.jaegertracing.io/) 的开源版本。目前 Zipkin 和 Jaeger 仍然是最流行的分布式链路追踪工具之一。
* 2015年 [OpenTracing](https://opentracing.io/) 项目被 CNCF 接受为它的第三个托管项目，致力于标准化跨组件的分布式链路追踪。
* 2017年 Google 将内部的 Census 项目开源，随后 [OpenCensus](https://opencensus.io/) 在社区中流行起来。
* 2017年 W3C 着手制定 [TraceContext](https://github.com/w3c/trace-context) 相关标准。
* 2019年初，两个现有开源项目：OpenTracing 和 OpenCensus 被宣布合并为 [OpenTelemetry](https://opentelemetry.io/) 项目，并将Log和Metrics。
* 2021年， OpenTelemetry 发布了V1.0.0，为客户端的链路追踪部分提供了稳定性保证。
* 2023年对于 OpenTelemetry 来说是一个里程碑，因为其三个基本信号，链路追踪、指标和日志，都达到了稳定版本。

{{ include(path="../../spring-opentelemetry/README.zh.md") }}