use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{ItemFn, Token};

use crate::input_and_compile_error;

struct ConfigArgs {
    route: bool,
    job: bool,
}

impl syn::parse::Parse for ConfigArgs {
    fn parse(args: syn::parse::ParseStream) -> syn::Result<Self> {
        let opts = args.parse_terminated(syn::MetaList::parse, Token![,])?;
        let mut route = false;
        let mut job = false;
        for meta in opts {
            if meta.path.is_ident("route") {
                route = true;
            } else if meta.path.is_ident("job") {
                job = true;
            } else {
                return Err(syn::Error::new_spanned(
                    meta.path,
                    "Unknown attribute key is specified; allowed: route, job and stream",
                ));
            }
        }
        Ok(ConfigArgs { route, job })
    }
}

struct AppConfig {
    args: ConfigArgs,
    ast: ItemFn,
}

impl AppConfig {
    fn new(args: ConfigArgs, ast: ItemFn) -> syn::Result<Self> {
        todo!()
    }
}

impl ToTokens for AppConfig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        todo!()
    }
}

pub(crate) fn config(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = match syn::parse(args) {
        Ok(config) => config,
        Err(e) => return input_and_compile_error(input, e),
    };

    let ast = match syn::parse::<syn::ItemFn>(input.clone()) {
        Ok(ast) => ast,
        // on parse error, make IDEs happy; see fn docs
        Err(err) => return input_and_compile_error(input, err),
    };

    match AppConfig::new(args, ast) {
        Ok(app_config) => app_config.into_token_stream().into(),
        // on macro related error, make IDEs happy; see fn docs
        Err(err) => input_and_compile_error(input, err),
    }
}
