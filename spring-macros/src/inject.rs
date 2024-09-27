use proc_macro2::{Span, TokenStream};
use quote::quote;

pub(crate) fn expand_derive(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    // let prefix = get_prefix(&input)?;
    let ident = input.ident;

    let output = quote! {
        impl ::spring::plugin::service::Service for #ident {
            fn build(app: &::spring::app::AppBuilder) -> ::spring::error::Result<Self> {
                Ok(Self{
                    
                })
            }
        }
    };

    Ok(output)
}