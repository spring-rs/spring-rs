//! Middleware that adds tracing to a [`Service`] that handles HTTP requests.

use http::{Request, Response};
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
use opentelemetry_semantic_conventions::attribute;
use pin_project::pin_project;
use std::{
    fmt::Display,
    future::Future,
    pin::Pin,
    task::{ready, Context, Poll},
};
use tower::{Layer, Service};
use tracing::{Level, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Describes the relationship between the [`Span`] and the service producing the span.
#[derive(Clone, Copy, Debug)]
enum SpanKind {
    /// The span describes a request sent to some remote service.
    Client,
    /// The span describes the server-side handling of a request.
    Server,
}

/// [`Layer`] that adds tracing to a [`Service`] that handles HTTP requests.
#[derive(Clone, Debug)]
pub struct HttpLayer {
    level: Level,
    kind: SpanKind,
    with_headers: bool,
    export_trace_id: bool,
}

impl HttpLayer {
    /// [`Span`]s are constructed at the given level from server side.
    pub fn server(level: Level) -> Self {
        Self {
            level,
            kind: SpanKind::Server,
            with_headers: true,
            export_trace_id: false,
        }
    }

    /// [`Span`]s are constructed at the given level from client side.
    pub fn client(level: Level) -> Self {
        Self {
            level,
            kind: SpanKind::Client,
            with_headers: true,
            export_trace_id: false,
        }
    }

    pub fn with_headers(mut self, with_headers: bool) -> Self {
        self.with_headers = with_headers;
        self
    }

    pub fn export_trace_id(mut self, export_trace_id: bool) -> Self {
        self.export_trace_id = export_trace_id;
        self
    }
}

impl<S> Layer<S> for HttpLayer {
    type Service = HttpService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HttpService {
            inner,
            level: self.level,
            kind: self.kind,
            with_headers: self.with_headers,
            export_trace_id: self.export_trace_id,
        }
    }
}

/// Middleware that adds tracing to a [`Service`] that handles HTTP requests.
#[derive(Clone, Debug)]
pub struct HttpService<S> {
    inner: S,
    level: Level,
    kind: SpanKind,
    with_headers: bool,
    export_trace_id: bool,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for HttpService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    S::Error: Display,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let span = self.make_request_span(&mut req);
        let inner = {
            let _enter = span.enter();
            self.inner.call(req)
        };

        ResponseFuture {
            inner,
            span,
            kind: self.kind,
            with_headers: self.with_headers,
            export_trace_id: self.export_trace_id,
        }
    }
}

impl<S> HttpService<S> {
    /// Creates a new [`Span`] for the given request.
    fn make_request_span<B>(&self, request: &mut Request<B>) -> Span {
        let Self { level, kind, .. } = self;

        macro_rules! make_span {
            ($level:expr) => {{
                use tracing::field::Empty;

                tracing::span!(
                    $level,
                    "HTTP",
                    "exception.message" = Empty,
                    "http.request.method" = tracing::field::display(request.method()),
                    "http.response.status_code" = Empty,
                    "network.protocol.name" = "http",
                    "network.protocol.version" = tracing::field::debug(request.version()),
                    "otel.kind" = tracing::field::debug(kind),
                    "otel.status_code" = Empty,
                    "url.full" = tracing::field::display(request.uri()),
                    "url.path" = request.uri().path(),
                    "url.query" = Empty,
                )
            }};
        }

        let span = match *level {
            Level::ERROR => make_span!(Level::ERROR),
            Level::WARN => make_span!(Level::WARN),
            Level::INFO => make_span!(Level::INFO),
            Level::DEBUG => make_span!(Level::DEBUG),
            Level::TRACE => make_span!(Level::TRACE),
        };

        if self.with_headers {
            for (header_name, header_value) in request.headers().iter() {
                if let Ok(attribute_value) = header_value.to_str() {
                    // attribute::HTTP_REQUEST_HEADER
                    let attribute_name = format!("http.request.header.{}", header_name);
                    span.set_attribute(attribute_name, attribute_value.to_owned());
                }
            }
        }

        if let Some(query) = request.uri().query() {
            span.record(attribute::URL_QUERY, query);
        }

        match kind {
            SpanKind::Client => {
                let context = span.context();
                opentelemetry::global::get_text_map_propagator(|injector| {
                    injector.inject_context(&context, &mut HeaderInjector(request.headers_mut()));
                });
            }
            SpanKind::Server => {
                let context = opentelemetry::global::get_text_map_propagator(|extractor| {
                    extractor.extract(&HeaderExtractor(request.headers()))
                });
                span.set_parent(context);
            }
        }

        span
    }
}

/// Response future for [`Http`].
#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    inner: F,
    span: Span,
    kind: SpanKind,
    with_headers: bool,
    export_trace_id: bool,
}

impl<F, ResBody, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
    E: Display,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _enter = this.span.enter();

        match ready!(this.inner.poll(cx)) {
            Ok(response) => {
                Self::record_response(this.span, *this.kind, *this.with_headers, &response);
                // if self.export_trace_id {
                //     response.headers_mut().append(key, Context::current().get().unwrap().s)
                // }
                Poll::Ready(Ok(response))
            }
            Err(err) => {
                Self::record_error(this.span, &err);
                Poll::Ready(Err(err))
            }
        }
    }
}

impl<F, ResBody, E> ResponseFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
    E: Display,
{
    /// Records fields associated to the response.
    fn record_response<B>(span: &Span, kind: SpanKind, with_headers: bool, response: &Response<B>) {
        span.record(
            attribute::HTTP_RESPONSE_STATUS_CODE,
            response.status().as_u16() as i64,
        );

        if with_headers {
            for (header_name, header_value) in response.headers().iter() {
                if let Ok(attribute_value) = header_value.to_str() {
                    let attribute_name = format!("http.response.header.{}", header_name);
                    span.set_attribute(attribute_name, attribute_value.to_owned());
                }
            }
        }

        if let SpanKind::Client = kind {
            if response.status().is_client_error() {
                span.record(attribute::OTEL_STATUS_CODE, "ERROR");
            }
        }
        if response.status().is_server_error() {
            span.record(attribute::OTEL_STATUS_CODE, "ERROR");
        }
    }

    /// Records the error message.
    fn record_error(span: &Span, err: &E) {
        span.record(attribute::OTEL_STATUS_CODE, "ERROR");
        span.record(attribute::EXCEPTION_MESSAGE, err.to_string());
    }
}
