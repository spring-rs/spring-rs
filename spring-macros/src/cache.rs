use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, ExprAssign, ItemFn, Lit, Token};

/// Cache arguments structure
struct CacheArgs {
    key: String,
    expire: Option<u64>,
}

impl syn::parse::Parse for CacheArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // key_pattern to match: "key:{key_id}"
        let key = input.parse::<syn::LitStr>().map_err(|mut err| {
            err.combine(syn::Error::new(
                err.span(),
                r#"invalid cache definition, expected #[cache("<key_pattern>", expire = <seconds>)]"#,
            ));

            err
        })?.value();

        // if there's no comma, assume that no options are provided
        if !input.peek(Token![,]) {
            return Ok(Self { key, expire: None });
        }

        // advance past comma separator
        input.parse::<Token![,]>()?;

        let mut expire = None;

        let assign = input.parse::<ExprAssign>()?;

        if let Expr::Path(path) = *assign.left {
            if path.path.is_ident("expire") {
                if let Expr::Lit(expr_lit) = *assign.right {
                    if let Lit::Int(lit_int) = expr_lit.lit {
                        expire = Some(lit_int.base10_parse()?);
                    } else {
                        return Err(syn::Error::new_spanned(
                            expr_lit,
                            "expire must be an integer",
                        ));
                    }
                }
            } else {
                return Err(syn::Error::new_spanned(path, "unknown named parameter"));
            }
        } else {
            return Err(syn::Error::new_spanned(
                assign.left,
                "invalid assignment key",
            ));
        }

        Ok(CacheArgs { key, expire })
    }
}

fn extract_ok_type_from_result(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        let segment = type_path.path.segments.last()?;
        if segment.ident == "Result" {
            if let syn::PathArguments::AngleBracketed(generic_args) = &segment.arguments {
                if let Some(syn::GenericArgument::Type(ok_ty)) = generic_args.args.first() {
                    return Some(ok_ty);
                }
            }
        }
    }
    None
}

pub fn cache(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let args = parse_macro_input!(attr as CacheArgs);

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let ident = &sig.ident;
    let inputs = &sig.inputs;
    let output = &sig.output;
    let asyncness = &sig.asyncness;
    let generics = &sig.generics;
    let where_clause = &sig.generics.where_clause;
    let attrs = &input_fn.attrs;
    let user_block = &input_fn.block;

    let ret_type = match &sig.output {
        syn::ReturnType::Type(_, ty) => ty,
        syn::ReturnType::Default => {
            return syn::Error::new_spanned(sig, "cached function must return a value")
                .to_compile_error()
                .into();
        }
    };

    let cache_key_fmt = args.key;
    let redis_set_stmt = match args.expire {
        Some(expire_sec) => {
            quote! {
                let _: ::std::result::Result<redis::Value, ()> = redis.set_ex(&cache_key, cache_value, #expire_sec).await
                    .map_err(|err| ::spring::tracing::error!("failed to set cache for key {}: {:?}", cache_key, err));
            }
        }
        None => {
            quote! {
                let _: ::std::result::Result<redis::Value, ()> = redis.set(&cache_key, cache_value).await
                    .map_err(|err| ::spring::tracing::error!("failed to set cache for key {}: {:?}", cache_key, err));
            }
        }
    };

    let gen_code = match extract_ok_type_from_result(ret_type) {
        Some(inner_type) => {
            quote! {
                #(#attrs)*
                #vis #asyncness fn #ident #generics(#inputs) #output #where_clause {
                    use spring_redis::redis::{self, AsyncCommands};
                    use spring::{plugin::ComponentRegistry, tracing, App};

                    let mut redis = App::global()
                        .get_component::<spring_redis::Redis>()
                        .expect("redis component not found");

                    let cache_key = format!(#cache_key_fmt);

                    if let Ok(Some(cache_value)) = redis.get::<_, Option<String>>(&cache_key).await {
                        match ::serde_json::from_str::<#inner_type>(&cache_value) {
                            Ok(value) => return Ok(value),
                            Err(e) => {
                                ::spring::tracing::error!("cache decode error for {}: {:?}", cache_key, e);
                            }
                        }
                    }

                    let result: #ret_type = (|| async #user_block)().await;
                    let result: #inner_type = result?;

                    match ::serde_json::to_string(&result) {
                        Ok(cache_value) => {
                            #redis_set_stmt
                        }
                        Err(err) => {
                            ::spring::tracing::error!("cache encode failed for key {}: {:?}", cache_key, err);
                        }
                    }

                    Ok(result)
                }
            }
        }
        None => {
            quote! {
                #(#attrs)*
                #vis #asyncness fn #ident #generics(#inputs) #output #where_clause {
                    use spring_redis::redis::{self, AsyncCommands};
                    use spring::{plugin::ComponentRegistry, App};

                    let mut redis = App::global()
                        .get_component::<spring_redis::Redis>()
                        .expect("redis component not found");

                    let cache_key = format!(#cache_key_fmt);

                    if let Ok(Some(cache_value)) = redis.get::<_, Option<String>>(&cache_key).await {
                        match ::serde_json::from_str::<#ret_type>(&cache_value) {
                            Ok(value) => return value,
                            Err(e) => {
                                ::spring::tracing::error!("cache decode error for {}: {:?}", cache_key, e);
                            }
                        }
                    }

                    let result: #ret_type = (|| async #user_block)().await;

                    match ::serde_json::to_string(&result) {
                        Ok(cache_value) => {
                            #redis_set_stmt
                        }
                        Err(err) => {
                            ::spring::tracing::error!("cache encode failed for key {}: {:?}", cache_key, err);
                        }
                    }

                    result
                }
            }
        }
    };

    gen_code.into()
}
