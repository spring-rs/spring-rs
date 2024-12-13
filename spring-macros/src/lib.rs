//! [![spring-rs](https://img.shields.io/github/stars/spring-rs/spring-rs)](https://spring-rs.github.io)
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]

mod auto;
mod config;
mod inject;
mod job;
mod nest;
mod route;
mod stream;

use proc_macro::TokenStream;
use syn::DeriveInput;

/// Creates resource handler, allowing multiple HTTP method guards.
///
/// # Syntax
/// ```plain
/// #[route("path", method="HTTP_METHOD"[, attributes])]
/// ```
///
/// # Attributes
/// - `"path"`: Raw literal string with path for which to register handler.
/// - `method = "HTTP_METHOD"`: Registers HTTP method to provide guard for. Upper-case string,
///   "GET", "POST" for example.
///
/// # Examples
/// ```
/// # use spring_web::axum::response::IntoResponse;
/// # use spring_macros::route;
/// #[route("/test", method = "GET", method = "HEAD")]
/// async fn example() -> impl IntoResponse {
///     "hello world"
/// }
/// ```
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    route::with_method(None, args, input)
}

/// Creates resource handler, allowing multiple HTTP methods and paths.
///
/// # Syntax
/// ```plain
/// #[routes]
/// #[<method>("path", ...)]
/// #[<method>("path", ...)]
/// ...
/// ```
///
/// # Attributes
/// The `routes` macro itself has no parameters, but allows specifying the attribute macros for
/// the multiple paths and/or methods, e.g. [`GET`](macro@get) and [`POST`](macro@post).
///
/// These helper attributes take the same parameters as the [single method handlers](crate#single-method-handler).
///
/// # Examples
/// ```
/// # use spring_web::axum::response::IntoResponse;
/// # use spring_macros::routes;
/// #[routes]
/// #[get("/test")]
/// #[get("/test2")]
/// #[delete("/test")]
/// async fn example() -> impl IntoResponse {
///     "hello world"
/// }
/// ```
#[proc_macro_attribute]
pub fn routes(_: TokenStream, input: TokenStream) -> TokenStream {
    route::with_methods(input)
}

macro_rules! method_macro {
    ($variant:ident, $method:ident) => {
        ///
        /// # Syntax
        /// ```plain
        #[doc = concat!("#[", stringify!($method), r#"("path"[, attributes])]"#)]
        /// ```
        ///
        /// # Attributes
        /// - `"path"`: Raw literal string with path for which to register handler.
        ///
        /// # Examples
        /// ```
        /// # use spring_web::axum::response::IntoResponse;
        #[doc = concat!("# use spring_macros::", stringify!($method), ";")]
        #[doc = concat!("#[", stringify!($method), r#"("/")]"#)]
        /// async fn example() -> impl IntoResponse {
        ///     "hello world"
        /// }
        /// ```
        #[proc_macro_attribute]
        pub fn $method(args: TokenStream, input: TokenStream) -> TokenStream {
            route::with_method(Some(route::Method::$variant), args, input)
        }
    };
}

method_macro!(Get, get);
method_macro!(Post, post);
method_macro!(Put, put);
method_macro!(Delete, delete);
method_macro!(Head, head);
method_macro!(Options, options);
method_macro!(Trace, trace);
method_macro!(Patch, patch);

/// Prepends a path prefix to all handlers using routing macros inside the attached module.
///
/// # Syntax
///
/// ```
/// # use spring_macros::nest;
/// #[nest("/prefix")]
/// mod api {
///     // ...
/// }
/// ```
///
/// # Arguments
///
/// - `"/prefix"` - Raw literal string to be prefixed onto contained handlers' paths.
///
/// # Example
///
/// ```
/// # use spring_macros::{nest, get};
/// # use spring_web::axum::response::IntoResponse;
/// #[nest("/api")]
/// mod api {
///     # use super::*;
///     #[get("/hello")]
///     pub async fn hello() -> impl IntoResponse {
///         // this has path /api/hello
///         "Hello, world!"
///     }
/// }
/// # fn main() {}
/// ```
#[proc_macro_attribute]
pub fn nest(args: TokenStream, input: TokenStream) -> TokenStream {
    nest::with_nest(args, input)
}

fn input_and_compile_error(mut item: TokenStream, err: syn::Error) -> TokenStream {
    let compile_err = TokenStream::from(err.to_compile_error());
    item.extend(compile_err);
    item
}

/// Job
///
macro_rules! job_macro {
    ($variant:ident, $job_type:ident, $example:literal) => {
        ///
        /// # Syntax
        /// ```plain
        #[doc = concat!("#[", stringify!($job_type), "(", $example, ")]")]
        /// ```
        ///
        /// # Attributes
        /// - `"path"`: Raw literal string with path for which to register handler.
        ///
        /// # Examples
        /// ```
        /// # use spring_web::axum::response::IntoResponse;
        #[doc = concat!("# use spring_macros::", stringify!($job_type), ";")]
        #[doc = concat!("#[", stringify!($job_type), "(", stringify!($example), ")]")]
        /// async fn example() {
        ///     println!("hello world");
        /// }
        /// ```
        #[proc_macro_attribute]
        pub fn $job_type(args: TokenStream, input: TokenStream) -> TokenStream {
            job::with_job(job::JobType::$variant, args, input)
        }
    };
}

job_macro!(OneShot, one_shot, 60);
job_macro!(FixDelay, fix_delay, 60);
job_macro!(FixRate, fix_rate, 60);
job_macro!(Cron, cron, "1/10 * * * * *");

/// Auto config
/// ```diff
///  use spring_macros::auto_config;
///  use spring_web::{WebPlugin, WebConfigurator};
///  use spring_job::{JobPlugin, JobConfigurator};
///  use spring_boot::app::App;
/// +#[auto_config(WebConfigurator, JobConfigurator)]
///  #[tokio::main]
///  async fn main() {
///      App::new()
///         .add_plugin(WebPlugin)
///         .add_plugin(JobPlugin)
/// -       .add_router(router())
/// -       .add_jobs(jobs())
///         .run()
///         .await
///  }
/// ```
///
#[proc_macro_attribute]
pub fn auto_config(args: TokenStream, input: TokenStream) -> TokenStream {
    auto::config(args, input)
}

/// stream macro
#[proc_macro_attribute]
pub fn stream_listener(args: TokenStream, input: TokenStream) -> TokenStream {
    stream::listener(args, input)
}

/// Configurable
#[proc_macro_derive(Configurable, attributes(config_prefix))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    config::expand_derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Injectable Servcie
#[proc_macro_derive(Service, attributes(prototype, inject))]
pub fn derive_service(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    inject::expand_derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
