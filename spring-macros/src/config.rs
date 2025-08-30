use proc_macro2::{Span, TokenStream};
use quote::quote;

pub(crate) fn expand_derive(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let prefix = get_prefix(&input)?;
    let ident = input.ident;

    let output = quote! {
        impl ::spring::config::Configurable for #ident {
            fn config_prefix() -> &'static str {
                    #prefix
            }
        }
        ::spring::submit_config!(#prefix, #ident);
    };

    Ok(output)
}

fn get_prefix(input: &syn::DeriveInput) -> syn::Result<syn::LitStr> {
    let attr = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("config_prefix"))
        .next_back();

    if let Some(syn::Attribute {
        meta: syn::Meta::NameValue(name_value),
        ..
    }) = attr
    {
        if name_value.path.is_ident("config_prefix") {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) = &name_value.value
            {
                return Ok(lit.clone());
            }
        }
    }
    Err(syn::Error::new(
        Span::call_site(),
        "missing attribute for Configurable, expected: #[config_prefix=\"prefix\"]",
    ))
}
