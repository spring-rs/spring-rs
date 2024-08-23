use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{punctuated::Punctuated, ItemFn, LitStr, MetaNameValue, Token};

use crate::input_and_compile_error;

enum ConsumerMode {
    /// This is the 'vanilla' stream consumer. It does not auto-commit, and thus only consumes messages from now on.
    RealTime,
    /// When the process restarts, it will resume the stream from the previous committed sequence.
    Resumable,
    /// You should assign a consumer group manually. The load-balancing mechanism is implementation-specific.
    LoadBalanced,
}

impl TryFrom<LitStr> for ConsumerMode {
    type Error = syn::Error;

    fn try_from(value: LitStr) -> Result<Self, Self::Error> {
        match value.value().as_str() {
            "RealTime" => Ok(Self::RealTime),
            "Resumable" => Ok(Self::Resumable),
            "LoadBalanced" => Ok(Self::LoadBalanced),
            _=>Err(syn::Error::new_spanned(value, "The optional modes are as follows: RealTime,Resumable,LoadBalanced. refs: https://docs.rs/sea-streamer/latest/sea_streamer/enum.ConsumerMode.html"))
        }
    }
}

#[derive(Default)]
struct StreamListenerArgs {
    topics: Vec<LitStr>,
    opts: ConsumerOpts,
}

impl syn::parse::Parse for StreamListenerArgs {
    fn parse(args: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut topics = Vec::<LitStr>::new();
        loop {
            let topic = args.parse::<LitStr>().map_err(|mut err| {
                err.combine(syn::Error::new(
                    err.span(),
                    r#"invalid stream definition, expected #[stream_listener("<topic>", "<topic>", mode="<mode>")]"#,
                ));

                err
            })?;

            topics.push(topic);

            if !args.peek(Token![,]) {
                return Ok(Self {
                    topics,
                    ..Default::default()
                });
            }
            args.parse::<Token![,]>()?;

            if args.cursor().literal().is_none() {
                break;
            }
        }

        let pairs = args.parse_terminated(syn::MetaNameValue::parse, Token![,])?;

        Ok(Self {
            topics,
            opts: ConsumerOpts::new(pairs)?,
        })
    }
}

#[derive(Default)]
struct ConsumerOpts {
    mode: Option<ConsumerMode>,
    group_id: Option<LitStr>,
    file_consumer_options: Option<LitStr>,
    stdio_consumer_options: Option<LitStr>,
    redis_consumer_options: Option<LitStr>,
    kafka_consumer_options: Option<LitStr>,
}

impl ConsumerOpts {
    fn new(pairs: Punctuated<MetaNameValue, Token![,]>) -> syn::Result<Self> {
        let mut mode = None;
        let mut group_id = None;
        let mut file_consumer_options = None;
        let mut stdio_consumer_options = None;
        let mut redis_consumer_options = None;
        let mut kafka_consumer_options = None;
        for pair in pairs {
            if pair.path.is_ident("mode") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) = pair.value
                {
                    mode = Some(ConsumerMode::try_from(lit)?);
                } else {
                    return Err(syn::Error::new_spanned(
                        pair.path,
                        r#"invalid stream definition, expected #[stream_listener("<topic>", mode="<mode>")]"#,
                    ));
                }
            } else if pair.path.is_ident("group_id") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) = pair.value
                {
                    group_id = Some(lit)
                } else {
                    return Err(syn::Error::new_spanned(
                        pair.path,
                        "group_id must be string literal",
                    ));
                }
            } else if pair.path.is_ident("file_consumer_options") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) = pair.value
                {
                    file_consumer_options = Some(lit)
                }
            } else if pair.path.is_ident("stdio_consumer_options") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) = pair.value
                {
                    stdio_consumer_options = Some(lit)
                }
            } else if pair.path.is_ident("redis_consumer_options") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) = pair.value
                {
                    redis_consumer_options = Some(lit)
                }
            } else if pair.path.is_ident("kafka_consumer_options") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) = pair.value
                {
                    kafka_consumer_options = Some(lit)
                }
            } else {
                return Err(syn::Error::new_spanned(
                    pair.path,
                    "Unknown attribute key is specified; allowed: mode,group_id,file_consumer_options,stdio_consumer_options,redis_consumer_options,kafka_consumer_options",
                ));
            }
        }
        Ok(Self {
            mode,
            group_id,
            file_consumer_options,
            stdio_consumer_options,
            redis_consumer_options,
            kafka_consumer_options,
        })
    }
}

pub(crate) struct StreamListener {
    /// Name of the handler function being annotated.
    name: syn::Ident,

    /// Args passed to stream_listener macro.
    args: StreamListenerArgs,

    /// AST of the handler function being annotated.
    ast: syn::ItemFn,

    /// The doc comment attributes to copy to generated struct, if any.
    doc_attributes: Vec<syn::Attribute>,
}

impl StreamListener {
    fn new(args: StreamListenerArgs, ast: ItemFn) -> syn::Result<Self> {
        let name = ast.sig.ident.clone();

        // Try and pull out the doc comments so that we can reapply them to the generated struct.
        // Note that multi line doc comments are converted to multiple doc attributes.
        let doc_attributes = ast
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .cloned()
            .collect();

        if ast.sig.asyncness.is_none() {
            return Err(syn::Error::new_spanned(
                ast.sig.fn_token,
                "only support async fn",
            ));
        }

        Ok(Self {
            name,
            args,
            ast,
            doc_attributes,
        })
    }
}

impl ToTokens for StreamListener {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        todo!()
    }
}

pub(crate) fn listener(args: TokenStream, input: TokenStream) -> TokenStream {
    eprintln!("{:#?}", args);
    let args: StreamListenerArgs = match syn::parse(args) {
        Ok(config) => config,
        Err(e) => return input_and_compile_error(input, e),
    };
    let ast = match syn::parse::<syn::ItemFn>(input.clone()) {
        Ok(ast) => ast,
        Err(err) => return input_and_compile_error(input, err),
    };
    match StreamListener::new(args, ast) {
        Ok(job) => job.into_token_stream().into(),
        Err(err) => input_and_compile_error(input, err),
    }
}
