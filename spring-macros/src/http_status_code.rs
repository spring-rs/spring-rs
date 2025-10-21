use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub(crate) fn expand_derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let data_enum = match &input.data {
        Data::Enum(data_enum) => data_enum,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "HttpStatusCode can only be derived for enums",
            ))
        }
    };

    let mut match_arms = Vec::new();
    let mut variant_info_arms = Vec::new();

    for variant in &data_enum.variants {
        let variant_ident = &variant.ident;
        let variant_name = variant_ident.to_string();

        let status_code_lit = get_status_code_literal(&variant.attrs)?;
        let status_code = quote! {
            ::spring_web::axum::http::StatusCode::from_u16(#status_code_lit).unwrap()
        };

        let pattern = match &variant.fields {
            Fields::Unit => quote! { #ident::#variant_ident },
            Fields::Unnamed(_) => quote! { #ident::#variant_ident(..) },
            Fields::Named(_) => quote! { #ident::#variant_ident { .. } },
        };

        match_arms.push(quote! {
            #pattern => #status_code
        });

        let is_transparent = has_transparent_attribute(&variant.attrs);

        let (description_str, inner_type_opt) = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field = fields.unnamed.first().unwrap();
                let inner_type = &field.ty;

                if is_transparent {
                    let inner_type_str = quote!(#inner_type).to_string().replace(' ', "");
                    (format!("{} (wraps {})", variant_name, inner_type_str), Some(inner_type.clone()))
                } else {
                    (format!("{} error", variant_name), Some(inner_type.clone()))
                }
            }
            _ => (format!("{} error{}", variant_name, if is_transparent { " (transparent)" } else { "" }), None)
        };

        let schema_gen = if let Some(inner_type) = inner_type_opt {
            if is_transparent {
                quote! {
                    {
                        let schema_opt: Option<::schemars::Schema> = None;
                        (
                            #status_code_lit,
                            #description_str.to_string(),
                            schema_opt
                        )
                    }
                }
            } else {
                quote! {
                    {
                        let mut gen = ::schemars::SchemaGenerator::default();
                        let schema_opt: Option<::schemars::Schema> = 
                            Some(<#inner_type as ::schemars::JsonSchema>::json_schema(&mut gen));
                        (
                            #status_code_lit,
                            #description_str.to_string(),
                            schema_opt
                        )
                    }
                }
            }
        } else {
            quote! {
                (#status_code_lit, #description_str.to_string(), None)
            }
        };

        variant_info_arms.push(quote! {
            #variant_name => Some(#schema_gen)
        });
    }

    let mod_name = quote::format_ident!("__http_status_code_impl_{}", ident.to_string().to_lowercase());
    
    let output = quote! {
        impl ::spring_web::HttpStatusCode for #ident {
            fn status_code(&self) -> ::spring_web::axum::http::StatusCode {
                match self {
                    #(#match_arms),*
                }
            }
        }

        impl ::spring_web::aide::OperationOutput for #ident {
            type Inner = Self;

            fn operation_response(
                _ctx: &mut ::spring_web::aide::generate::GenContext,
                _operation: &mut ::spring_web::aide::openapi::Operation,
            ) -> Option<::spring_web::aide::openapi::Response> {
                None
            }

            fn inferred_responses(
                _ctx: &mut ::spring_web::aide::generate::GenContext,
                _operation: &mut ::spring_web::aide::openapi::Operation,
            ) -> Vec<(Option<u16>, ::spring_web::aide::openapi::Response)> {
                vec![]
            }
        }

        #[doc(hidden)]
        mod #mod_name {
            use super::*;
            
            pub fn get_variant_info(variant_name: &str) -> Option<(u16, String, Option<::schemars::Schema>)> {
                match variant_name {
                    #(#variant_info_arms,)*
                    _ => None
                }
            }
        }

        impl #ident {
            #[doc(hidden)]
            pub fn __get_variant_info(variant_name: &str) -> Option<(u16, String, Option<::schemars::Schema>)> {
                #mod_name::get_variant_info(variant_name)
            }
        }

        impl ::spring_web::openapi::HttpStatusCodeVariantInfo for #ident {
            fn get_variant_info(variant_name: &str) -> Option<(u16, String, Option<::schemars::Schema>)> {
                Self::__get_variant_info(variant_name)
            }
        }
    };

    Ok(output)
}

fn get_status_code_literal(attrs: &[syn::Attribute]) -> syn::Result<u16> {
    for attr in attrs {
        if attr.path().is_ident("status_code") {
            // Parse the attribute value
            let status_code: syn::LitInt = attr.parse_args()?;
            return status_code.base10_parse::<u16>();
        }
    }

    Err(syn::Error::new_spanned(
        attrs.first(),
        "Missing #[status_code(...)] attribute. Each variant must have a status_code attribute, e.g., #[status_code(404)]",
    ))
}

fn has_transparent_attribute(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("error") {
            if let Ok(meta) = attr.parse_args::<syn::Ident>() {
                if meta == "transparent" {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_derive() {
        let input: DeriveInput = syn::parse_quote! {
            #[derive(HttpStatusCode)]
            pub enum Errors {
                #[status_code(401)]
                A,
                #[status_code(403)]
                B,
            }
        };

        let result = expand_derive(input).unwrap();
        // Just check it compiles without panicking
        assert!(!result.is_empty());
    }
}
