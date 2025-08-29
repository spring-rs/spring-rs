//! [![spring-rs](https://img.shields.io/github/stars/spring-rs/spring-rs)](https://spring-rs.github.io)
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]
#![allow(clippy::needless_doctest_main)]

mod auto;
mod cache;
mod config;
mod inject;
mod job;
mod middlewares;
mod nest;
mod route;
mod stream;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

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

/// Applies middleware layers to all route handlers within a module.
///
/// # Syntax
/// ```plain
/// #[middlewares(middleware1, middleware2, ...)]
/// mod module_name {
///     // route handlers
/// }
/// ```
///
/// # Arguments
/// - `middleware1`, `middleware2`, etc. - Middleware expressions that will be applied to all routes in the module
///
/// This macro generates a router function that applies the specified middleware
/// to all route handlers defined within the module.
#[proc_macro_attribute]
pub fn middlewares(args: TokenStream, input: TokenStream) -> TokenStream {
    middlewares::middlewares(args, input)
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
#[proc_macro_derive(Service, attributes(service, inject))]
pub fn derive_service(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    inject::expand_derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// `#[cache]` - Transparent Redis-based caching for async functions.
///
/// This macro wraps an async function to automatically cache its result
/// in Redis. It checks for a cached value before executing the function.
/// If a cached result is found, it is deserialized and returned directly.
/// Otherwise, the function runs normally and its result is stored in Redis.
///
/// # Syntax
/// ```plain
/// #[cache("key_pattern", expire = <seconds>, condition = <bool_expr>, unless = <bool_expr>)]
/// ```
///
/// # Attributes
/// - `"key_pattern"` (**required**):
///   A format string used to generate the cache key. Function arguments can be interpolated using standard `format!` syntax.
/// - `expire = <integer>` (**optional**):
///   The number of seconds before the cached value expires. If omitted, the key will be stored without expiration.
/// - `condition = <expression>` (**optional**):
///   A boolean expression evaluated **before** executing the function.
///   If this evaluates to `false`, caching is completely bypassed — no lookup and no insertion.
///   The expression can access function parameters directly.
/// - `unless = <expression>` (**optional**):
///   A boolean expression evaluated **after** executing the function.
///   If this evaluates to `true`, the result will **not** be written to the cache.
///   The expression can access both parameters and a `result` variable (the return value).
///   NOTE: If your function returns Result<T, E>, the `result` variable in unless refers to the inner Ok value (T), not the entire Result.
///   This allows you to write expressions like result.is_none() for Result<Option<_>, _> functions.
///
/// # Function Requirements
/// - Must be an `async fn`
/// - Can return either a `Result<T, E>` or a plain value `T`
/// - The return type must implement `serde::Serialize` and `serde::Deserialize`
/// - Generics, attributes, and visibility will be preserved
///
/// # Example
/// ```rust
/// use spring_macros::cache;
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct User {
///     id: u64,
///     name: String,
/// }
///
/// struct MyError;
///
/// #[cache("user:{user_id}", expire = 600, condition = user_id % 2 == 0, unless = result.is_none())]
/// async fn get_user(user_id: u64) -> Result<Option<User>, MyError> {
///     // Fetch user from database
///     unimplemented!("do something")
/// }
/// ```
#[proc_macro_attribute]
pub fn cache(args: TokenStream, input: TokenStream) -> TokenStream {
    cache::cache(args, input)
}



/// ### This is just a re-export of `tokio::main` to simplify the initial setup of
/// ### spring-rs, you can use your own version of tokio adding it to your 
/// ### `Cargo.toml`, we are providing this initial base by default
/// 
/// ---
/// 
/// Marks async function to be executed by the selected runtime. This macro
/// helps set up a `Runtime` without requiring the user to use
/// [Runtime](../tokio/runtime/struct.Runtime.html) or
/// [Builder](../tokio/runtime/struct.Builder.html) directly.
///
/// Note: This macro is designed to be simplistic and targets applications that
/// do not require a complex setup. If the provided functionality is not
/// sufficient, you may be interested in using
/// [Builder](../tokio/runtime/struct.Builder.html), which provides a more
/// powerful interface.
///
/// Note: This macro can be used on any function and not just the `main`
/// function. Using it on a non-main function makes the function behave as if it
/// was synchronous by starting a new runtime each time it is called. If the
/// function is called often, it is preferable to create the runtime using the
/// runtime builder so the runtime can be reused across calls.
///
/// # Non-worker async function
///
/// Note that the async function marked with this macro does not run as a
/// worker. The expectation is that other tasks are spawned by the function here.
/// Awaiting on other futures from the function provided here will not
/// perform as fast as those spawned as workers.
///
/// # Multi-threaded runtime
///
/// To use the multi-threaded runtime, the macro can be configured using
///
/// ```
/// #[tokio::main(flavor = "multi_thread", worker_threads = 10)]
/// # async fn main() {}
/// ```
///
/// The `worker_threads` option configures the number of worker threads, and
/// defaults to the number of cpus on the system. This is the default flavor.
///
/// Note: The multi-threaded runtime requires the `rt-multi-thread` feature
/// flag.
///
/// # Current thread runtime
///
/// To use the single-threaded runtime known as the `current_thread` runtime,
/// the macro can be configured using
///
/// ```
/// #[tokio::main(flavor = "current_thread")]
/// # async fn main() {}
/// ```
///
/// ## Function arguments:
///
/// Arguments are allowed for any functions aside from `main` which is special
///
/// ## Usage
///
/// ### Using the multi-thread runtime
///
/// ```rust
/// #[tokio::main]
/// async fn main() {
///     println!("Hello world");
/// }
/// ```
///
/// Equivalent code not using `#[tokio::main]`
///
/// ```rust
/// fn main() {
///     tokio::runtime::Builder::new_multi_thread()
///         .enable_all()
///         .build()
///         .unwrap()
///         .block_on(async {
///             println!("Hello world");
///         })
/// }
/// ```
///
/// ### Using current thread runtime
///
/// The basic scheduler is single-threaded.
///
/// ```rust
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() {
///     println!("Hello world");
/// }
/// ```
///
/// Equivalent code not using `#[tokio::main]`
///
/// ```rust
/// fn main() {
///     tokio::runtime::Builder::new_current_thread()
///         .enable_all()
///         .build()
///         .unwrap()
///         .block_on(async {
///             println!("Hello world");
///         })
/// }
/// ```
///
/// ### Set number of worker threads
///
/// ```rust
/// #[tokio::main(worker_threads = 2)]
/// async fn main() {
///     println!("Hello world");
/// }
/// ```
///
/// Equivalent code not using `#[tokio::main]`
///
/// ```rust
/// fn main() {
///     tokio::runtime::Builder::new_multi_thread()
///         .worker_threads(2)
///         .enable_all()
///         .build()
///         .unwrap()
///         .block_on(async {
///             println!("Hello world");
///         })
/// }
/// ```
///
/// ### Configure the runtime to start with time paused
///
/// ```rust
/// #[tokio::main(flavor = "current_thread", start_paused = true)]
/// async fn main() {
///     println!("Hello world");
/// }
/// ```
///
/// Equivalent code not using `#[tokio::main]`
///
/// ```rust
/// fn main() {
///     tokio::runtime::Builder::new_current_thread()
///         .enable_all()
///         .start_paused(true)
///         .build()
///         .unwrap()
///         .block_on(async {
///             println!("Hello world");
///         })
/// }
/// ```
///
/// Note that `start_paused` requires the `test-util` feature to be enabled.
///
/// ### Rename package
///
/// ```rust
/// use tokio as tokio1;
///
/// #[tokio1::main(crate = "tokio1")]
/// async fn main() {
///     println!("Hello world");
/// }
/// ```
///
/// Equivalent code not using `#[tokio::main]`
///
/// ```rust
/// use tokio as tokio1;
///
/// fn main() {
///     tokio1::runtime::Builder::new_multi_thread()
///         .enable_all()
///         .build()
///         .unwrap()
///         .block_on(async {
///             println!("Hello world");
///         })
/// }
/// ```
///
/// ### Configure unhandled panic behavior
///
/// Available options are `shutdown_runtime` and `ignore`. For more details, see
/// [`Builder::unhandled_panic`].
///
/// This option is only compatible with the `current_thread` runtime.
///
/// ```no_run
/// # #![allow(unknown_lints, unexpected_cfgs)]
/// #[cfg(tokio_unstable)]
/// #[tokio::main(flavor = "current_thread", unhandled_panic = "shutdown_runtime")]
/// async fn main() {
///     let _ = tokio::spawn(async {
///         panic!("This panic will shutdown the runtime.");
///     }).await;
/// }
/// # #[cfg(not(tokio_unstable))]
/// # fn main() { }
/// ```
///
/// Equivalent code not using `#[tokio::main]`
///
/// ```no_run
/// # #![allow(unknown_lints, unexpected_cfgs)]
/// #[cfg(tokio_unstable)]
/// fn main() {
///     tokio::runtime::Builder::new_current_thread()
///         .enable_all()
///         .unhandled_panic(UnhandledPanic::ShutdownRuntime)
///         .build()
///         .unwrap()
///         .block_on(async {
///             let _ = tokio::spawn(async {
///                 panic!("This panic will shutdown the runtime.");
///             }).await;
///         })
/// }
/// # #[cfg(not(tokio_unstable))]
/// # fn main() { }
/// ```
///
/// **Note**: This option depends on Tokio's [unstable API][unstable]. See [the
/// documentation on unstable features][unstable] for details on how to enable
/// Tokio's unstable features.
///
/// [`Builder::unhandled_panic`]: ../tokio/runtime/struct.Builder.html#method.unhandled_panic
/// [unstable]: ../tokio/index.html#unstable-features
#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);
    let fn_body = input.block;
    let fn_attrs = input.attrs;
    let fn_vis = input.vis;
    let _fn_sig = input.sig;

    let output = if cfg!(feature = "hot-reload") {
        quote! {
            async fn __internal_main() {
                #fn_body
            }

            #(#fn_attrs)*
            #fn_vis fn main() {
                ::tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Failed building the Runtime")
                    .block_on(dioxus_devtools::serve_subsecond(__internal_main));
            }
        }
    } else {
        quote! {
            #(#fn_attrs)*
            #fn_vis fn main() {
                ::tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Failed building the Runtime")
                    .block_on( async #fn_body );
            }
        }
    };
    output.into()
}