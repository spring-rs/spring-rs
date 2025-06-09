use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, Expr, Token, parse::Parse};

struct MiddlewareList {
    middlewares: Punctuated<Expr, Token![,]>,
}

impl Parse for MiddlewareList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let middlewares = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
        Ok(MiddlewareList { middlewares })
    }
}

pub fn middlewares(args: TokenStream, input: TokenStream) -> TokenStream {
    match middlewares_inner(args, input.clone()) {
        Ok(stream) => stream,
        Err(err) => crate::input_and_compile_error(input, err),
    }
}

fn middlewares_inner(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    if args.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "missing arguments for middlewares macro, expected: #[middlewares(middleware1, middleware2, ...)]",
        ));
    }

    let middleware_list = syn::parse::<MiddlewareList>(args.clone()).map_err(|err| {
        syn::Error::new(
            err.span(),
            "arguments to middlewares macro must be valid expressions, expected: #[middlewares(middleware1, middleware2, ...)]",
        )
    })?;

    let mut module = syn::parse::<syn::ItemMod>(input).map_err(|err| {
        syn::Error::new(err.span(), "#[middlewares] macro must be attached to a module")
    })?;

    let module_name = &module.ident;
    let registrar_struct_name = syn::Ident::new(&format!("{}MiddlewareRegistrar", module_name), module.ident.span());

    let nest_prefix = extract_nest_prefix(&module)?;

    let route_info = collect_and_strip_route_info(&mut module, nest_prefix.as_deref())?;

    let (route_registrations, middleware_expressions) = generate_middleware_components(
        &middleware_list.middlewares,
        &route_info,
    )?;

    if let Some((_, ref mut items)) = module.content {
        let registrar_struct: syn::ItemStruct = syn::parse2(quote! {
            #[allow(non_camel_case_types, missing_docs)]
            struct #registrar_struct_name;
        })?;
        
        let registrar_impl: syn::ItemImpl = syn::parse2(quote! {
            impl ::spring_web::handler::TypedHandlerRegistrar for #registrar_struct_name {
                fn install_route(&self, mut __router: ::spring_web::Router) -> ::spring_web::Router {
                    use ::spring_web::handler::TypeRouter;
                    
                    let mut __module_router = ::spring_web::Router::new();
                    
                    #(#route_registrations)*
                    
                    #(let __module_router = __module_router.layer(#middleware_expressions);)*
                    
                    __router = __router.merge(__module_router);
                    
                    __router
                }
            }
        })?;
        
        let submit_call: syn::ItemMacro = syn::parse2(quote! {
            ::spring_web::submit_typed_handler!(#registrar_struct_name);
        })?;
        
        items.push(syn::Item::Struct(registrar_struct));
        items.push(syn::Item::Impl(registrar_impl));
        items.push(syn::Item::Macro(submit_call));
    } else {
        return Err(syn::Error::new(
            module.ident.span(),
            "Module must have content to apply middlewares",
        ));
    }

    Ok(module.into_token_stream().into())
}

struct RouteInfo {
    func_name: syn::Ident,
    path: String,
    methods: Vec<String>,
}

fn collect_and_strip_route_info(module: &mut syn::ItemMod, nest_prefix: Option<&str>) -> syn::Result<Vec<RouteInfo>> {
    let mut route_info = Vec::new();
    
    if let Some((_, items)) = &mut module.content {
        for item in items {
            if let syn::Item::Fn(fun) = item {
                let mut route_attrs = Vec::new();
                fun.attrs.retain(|attr| {
                    let is_route_attr = attr.path().is_ident("get") ||
                        attr.path().is_ident("post") ||
                        attr.path().is_ident("put") ||
                        attr.path().is_ident("delete") ||
                        attr.path().is_ident("patch") ||
                        attr.path().is_ident("head") ||
                        attr.path().is_ident("options") ||
                        attr.path().is_ident("trace") ||
                        attr.path().is_ident("route") ||
                        attr.path().is_ident("routes");
                    
                    if is_route_attr {
                        route_attrs.push(attr.clone());
                        false
                    } else {
                        true
                    }
                });
                
                for attr in route_attrs {
                    let method = if attr.path().is_ident("get") {
                        "GET"
                    } else if attr.path().is_ident("post") {
                        "POST"
                    } else if attr.path().is_ident("put") {
                        "PUT"
                    } else if attr.path().is_ident("delete") {
                        "DELETE"
                    } else if attr.path().is_ident("patch") {
                        "PATCH"
                    } else if attr.path().is_ident("head") {
                        "HEAD"
                    } else if attr.path().is_ident("options") {
                        "OPTIONS"
                    } else if attr.path().is_ident("trace") {
                        "TRACE"
                    } else {
                        continue; 
                    };
                    
                    if let Ok(path_lit) = attr.parse_args::<syn::LitStr>() {
                        let path = if let Some(prefix) = nest_prefix {
                            format!("{}{}", prefix, path_lit.value())
                        } else {
                            path_lit.value()
                        };
                        
                        route_info.push(RouteInfo {
                            func_name: fun.sig.ident.clone(),
                            path,
                            methods: vec![method.to_string()],
                        });
                    }
                }
            }
        }
    }

    Ok(route_info)
}

fn generate_middleware_components(
    middleware_list: &Punctuated<Expr, Token![,]>,
    route_info: &[RouteInfo],
) -> syn::Result<(Vec<TokenStream2>, Vec<TokenStream2>)> {
    let middleware_expressions: Vec<TokenStream2> = middleware_list
        .iter()
        .map(|middleware| {
            quote! { #middleware }
        })
        .collect();

    let route_registrations: Vec<TokenStream2> = route_info
        .iter()
        .map(|route| {
            let func_name = &route.func_name;
            let path = &route.path;
            let methods: Vec<TokenStream2> = route.methods
                .iter()
                .map(|method_str| {
                    let method_ident = syn::Ident::new(method_str, Span::call_site());
                    quote! { ::spring_web::MethodFilter::#method_ident }
                })
                .collect();
                
            quote! { 
                let __method_router = ::spring_web::MethodRouter::new();
                #(let __method_router = ::spring_web::MethodRouter::on(__method_router, #methods, #func_name);)*
                __module_router = ::spring_web::Router::route(__module_router, #path, __method_router);
            }
        })
        .collect();

    Ok((route_registrations, middleware_expressions))
}

fn extract_nest_prefix(module: &syn::ItemMod) -> syn::Result<Option<String>> {
    for attr in &module.attrs {
        if attr.path().is_ident("nest") {
            if let Ok(path_lit) = attr.parse_args::<syn::LitStr>() {
                return Ok(Some(path_lit.value()));
            }
        }
    }
    Ok(None)
}
