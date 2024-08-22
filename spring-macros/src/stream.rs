use proc_macro::TokenStream;
use syn::{punctuated::Punctuated, Ident, LitStr, MetaNameValue, Token};

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
        // Self::parse(value.value().as_str())
        //     .map_err(|message| syn::Error::new_spanned(value, message))
        Ok(Self::RealTime)
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
                    r#"invalid stream definition, expected #[stream_listener("<topic>", "<topic>", ..)]"#,
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
    file_consumer_options: Option<Ident>,
    stdio_consumer_options: Option<Ident>,
    redis_consumer_options: Option<Ident>,
    kafka_consumer_options: Option<Ident>,
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
                }
            } else if pair.path.is_ident("group_id") {
            } else if pair.path.is_ident("file_consumer_options") {
            } else if pair.path.is_ident("stdio_consumer_options") {
            } else if pair.path.is_ident("redis_consumer_options") {
            } else if pair.path.is_ident("kafka_consumer_options") {
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

pub(crate) fn listener(args: TokenStream, input: TokenStream) -> TokenStream {
    eprintln!("{:#?}", args);
    let args: StreamListenerArgs = match syn::parse(args) {
        Ok(config) => config,
        Err(e) => return input_and_compile_error(input, e),
    };
    input
}
