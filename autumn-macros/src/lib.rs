//! Routing and runtime macros for autumn.rs.
//!
//! # Autumn.rs Re-exports
//! Autumn.rs re-exports a version of this crate in it's entirety so you usually don't have to
//! specify a dependency on this crate explicitly. Sometimes, however, updates are made to this
//! crate before the Autumn.rs dependency is updated. Therefore, code examples here will show
//! explicit imports. Check the latest [Autumn.rs attributes docs] to see which macros
//! are re-exported.
//!
//! # Runtime Setup
//! Used for setting up the Autumn.rs async runtime. See [macro@main] macro docs.
//!
//! ```
//! #[autumn_macros::main] // or `#[autumn::main]` in Autumn.rs Web apps
//! async fn main() {
//!     App::new().run().await
//! }
//! ```
//!
//! # Single Method Handler
//! There is a macro to set up a handler for each of the most common HTTP methods that also define
//! additional guards and route-specific middleware.
//!
//! See docs for: [GET], [POST], [PATCH], [PUT], [DELETE], [HEAD], [CONNECT], [OPTIONS], [TRACE]
//!
//! ```
//! # use autumn_web::response::IntoResponse;
//! # use autumn_macros::get;
//! #[get("/test")]
//! async fn get_handler() -> impl IntoResponse {
//!     "hello world"
//! }
//! ```
//!
//! # Multiple Method Handlers
//! Similar to the single method handler macro but takes one or more arguments for the HTTP methods
//! it should respond to. See [macro@route] macro docs.
//!
//! ```
//! # use autumn_web::response::IntoResponse;
//! # use autumn_macros::route;
//! #[route("/test", method = "GET", method = "HEAD")]
//! async fn get_and_head_handler() -> impl IntoResponse {
//!     "hello world"
//! }
//! ```
//!
//! # Multiple Path Handlers
//! Acts as a wrapper for multiple single method handler macros. It takes no arguments and
//! delegates those to the macros for the individual methods. See [macro@routes] macro docs.
//!
//! ```
//! # use autumn_web::response::IntoResponse;
//! # use autumn_macros::routes;
//! #[routes]
//! #[get("/test")]
//! #[get("/test2")]
//! #[delete("/test")]
//! async fn example() -> impl IntoResponse {
//!     "hello world"
//! }
//! ```

mod nest;
mod route;

use proc_macro::TokenStream;

/// Creates resource handler, allowing multiple HTTP method guards.
///
/// # Syntax
/// ```plain
/// #[route("path", method="HTTP_METHOD"[, attributes])]
/// ```
///
/// # Attributes
/// - `"path"`: Raw literal string with path for which to register handler.
/// - `name = "resource_name"`: Specifies resource name for the handler. If not set, the function
///   name of handler is used.
/// - `method = "HTTP_METHOD"`: Registers HTTP method to provide guard for. Upper-case string,
///   "GET", "POST" for example.
/// - `guard = "function_name"`: Registers function as guard using `actix_web::guard::fn_guard`.
/// - `wrap = "Middleware"`: Registers a resource middleware.
///
/// # Notes
/// Function name can be specified as any expression that is going to be accessible to the generate
/// code, e.g `my_guard` or `my_module::my_guard`.
///
/// # Examples
/// ```
/// # use autumn_web::response::IntoResponse;
/// # use autumn_macros::route;
/// #[route("/test", method = "GET", method = "HEAD", method = "CUSTOM")]
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
/// # use autumn_web::response::IntoResponse;
/// # use autumn_macros::routes;
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
        /// # use autumn_web::response::IntoResponse;
        #[doc = concat!("# use autumn_macros::", stringify!($method), ";")]
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
/// # use autumn_macros::nest;
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
/// # use autumn_macros::{nest, get};
/// # use autumn_web::response::IntoResponse;
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

/// Marks async main function as the autumn.rs system entry-point.
///
/// # Examples
/// ```
/// #[autumn::main]
/// async fn main() {
///     App::new().run().await
/// }
/// ```
#[proc_macro_attribute]
pub fn main(_: TokenStream, item: TokenStream) -> TokenStream {
    // let mut output: TokenStream = (quote! {
    //     #[::actix_web::rt::main(system = "::actix_web::rt::System")]
    // })
    // .into();

    // output.extend(item);
    // output
    item
}

fn input_and_compile_error(mut item: TokenStream, err: syn::Error) -> TokenStream {
    let compile_err = TokenStream::from(err.to_compile_error());
    item.extend(compile_err);
    item
}
