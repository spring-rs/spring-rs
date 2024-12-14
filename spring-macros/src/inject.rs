use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    spanned::Spanned, AngleBracketedGenericArguments, GenericArgument, Meta, MetaList,
    PathArguments, Token, Type, TypePath,
};

fn inject_error_tip() -> syn::Error {
    syn::Error::new(
        Span::call_site(),
        "inject Service only support Named-field Struct",
    )
}

enum InjectableType {
    Option,
    Component(syn::Path),
    Config(syn::Path),
    ComponentRef(syn::Path),
    ConfigRef(syn::Path),
    FuncCall(syn::ExprCall),
    PrototypeArg(syn::Path),
}

impl InjectableType {
    fn order(&self) -> u8 {
        match self {
            Self::Option => 0,
            Self::Component(_) => 1,
            Self::Config(_) => 2,
            Self::ComponentRef(_) => 3,
            Self::ConfigRef(_) => 4,
            Self::FuncCall(_) => 5,
            Self::PrototypeArg(_) => 6,
        }
    }

    fn is_arg(&self) -> bool {
        matches!(self, Self::PrototypeArg(_))
    }
}

enum InjectableAttr {
    Component,
    Config,
    FuncCall(syn::ExprCall),
}

struct Injectable {
    is_prototype: bool,
    ty: InjectableType,
    field_name: syn::Ident,
}

impl Injectable {
    fn new(field: syn::Field, is_prototype: bool) -> syn::Result<Self> {
        let ty = Self::compute_type(&field, is_prototype)?;
        let field_name = field.ident.ok_or_else(inject_error_tip)?;
        Ok(Self {
            is_prototype,
            ty,
            field_name,
        })
    }

    fn compute_type(field: &syn::Field, is_prototype: bool) -> syn::Result<InjectableType> {
        let ty = if let syn::Type::Path(path) = &field.ty {
            &path.path
        } else {
            Err(inject_error_tip())?
        };
        let inject_attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("inject"));

        if let Some(inject_attr) = inject_attr {
            if let Meta::List(MetaList { tokens, .. }) = &inject_attr.meta {
                let attr = syn::parse::<InjectableAttr>(tokens.clone().into())?;
                return Ok(attr.make_type(ty));
            } else {
                Err(syn::Error::new_spanned(
                    inject_attr,
                    "invalid inject definition, expected #[inject(component|config|func(args))]",
                ))?;
            }
        }
        let last_path_segment = ty.segments.last().ok_or_else(inject_error_tip)?;
        if last_path_segment.ident == "ComponentRef" {
            return Ok(InjectableType::ComponentRef(Self::get_argument_type(
                &last_path_segment.arguments,
            )?));
        }
        if last_path_segment.ident == "ConfigRef" {
            return Ok(InjectableType::ConfigRef(Self::get_argument_type(
                &last_path_segment.arguments,
            )?));
        }
        if is_prototype {
            Ok(InjectableType::PrototypeArg(ty.clone()))
        } else {
            if last_path_segment.ident == "Option" {
                return Ok(InjectableType::Option);
            }
            let field_name = &field
                .ident
                .clone()
                .map(|ident| ident.to_string())
                .ok_or_else(inject_error_tip)?;
            Err(syn::Error::new_spanned(
            field,
            format!(
                "{field_name} field missing inject definition, expected #[inject(component|config|func(args))]",
            ),
        ))
        }
    }

    fn get_argument_type(path_args: &PathArguments) -> syn::Result<syn::Path> {
        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
            path_args
        {
            let ty = args.last().ok_or_else(inject_error_tip)?;
            if let GenericArgument::Type(Type::Path(TypePath { path, .. })) = ty {
                return Ok(path.clone());
            }
        }
        Err(inject_error_tip())
    }
}

impl syn::parse::Parse for InjectableAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Path>()?;
        if name.is_ident("component") {
            return Ok(Self::Component);
        }
        if name.is_ident("config") {
            return Ok(Self::Config);
        }
        if name.is_ident("func") {
            input.parse::<Token![=]>()?;
            let func_call = input.parse::<syn::ExprCall>()?;
            return Ok(Self::FuncCall(func_call));
        }
        Err(syn::Error::new(
            Span::call_site(),
            "invalid inject definition, expected #[inject(component|config|func(args))]",
        ))
    }
}

impl InjectableAttr {
    fn make_type(self, ty: &syn::Path) -> InjectableType {
        match self {
            Self::Component => InjectableType::Component(ty.clone()),
            Self::Config => InjectableType::Config(ty.clone()),
            Self::FuncCall(func_call) => InjectableType::FuncCall(func_call),
        }
    }
}

impl ToTokens for Injectable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            is_prototype,
            ty,
            field_name,
        } = self;
        match ty {
            InjectableType::Option => {
                tokens.extend(quote! {
                    let #field_name = None;
                });
            }
            InjectableType::Component(type_path) => {
                if *is_prototype {
                    tokens.extend(quote! {
                        let #field_name = ::spring::App::global().try_get_component::<#type_path>()?;
                    });
                } else {
                    tokens.extend(quote! {
                        let #field_name = app.try_get_component::<#type_path>()?;
                    });
                }
            }
            InjectableType::Config(type_path) => {
                if *is_prototype {
                    tokens.extend(quote! {
                        let #field_name = ::spring::App::global().get_config::<#type_path>()?;
                    });
                } else {
                    tokens.extend(quote! {
                        let #field_name = app.get_config::<#type_path>()?;
                    });
                }
            }
            InjectableType::ComponentRef(type_path) => {
                if *is_prototype {
                    tokens.extend(quote! {
                        let #field_name = ::spring::App::global().try_get_component_ref::<#type_path>()?;
                    });
                } else {
                    tokens.extend(quote! {
                        let #field_name = app.try_get_component_ref::<#type_path>()?;
                    });
                }
            }
            InjectableType::ConfigRef(type_path) => {
                if *is_prototype {
                    tokens.extend(quote! {
                        let #field_name = ::spring::config::ConfigRef::new(::spring::App::global().get_config::<#type_path>()?);
                    });
                } else {
                    tokens.extend(quote! {
                        let #field_name = ::spring::config::ConfigRef::new(app.get_config::<#type_path>()?);
                    });
                }
            }
            InjectableType::FuncCall(func_call) => {
                tokens.extend(quote! {
                    let #field_name = #func_call;
                });
            }
            InjectableType::PrototypeArg(type_path) => {
                // as func args
                tokens.extend(quote! {
                    #field_name: #type_path
                });
            }
        }
    }
}

struct Service {
    ident: proc_macro2::Ident,
    prototype: Option<Prototype>,
    fields: Vec<Injectable>,
}

impl Service {
    fn new(input: syn::DeriveInput) -> syn::Result<Self> {
        let syn::DeriveInput {
            ident, attrs, data, ..
        } = input;
        let prototype = attrs.iter().find(|a| a.path().is_ident("prototype"));
        let prototype = match prototype {
            Some(attr) => Some(Prototype::new(attr)?),
            None => None,
        };
        let is_prototype = prototype.is_some();
        let mut fields = if let syn::Data::Struct(data) = data {
            data.fields
                .into_iter()
                .map(|f| Injectable::new(f, is_prototype))
                .collect::<syn::Result<Vec<_>>>()?
        } else {
            return Err(inject_error_tip());
        };
        fields.sort_by_key(|f| f.ty.order());

        // Put FuncCall at the end
        Ok(Self {
            ident,
            prototype,
            fields,
        })
    }
}

struct Prototype {
    build: syn::LitStr,
}

impl Prototype {
    fn new(attr: &syn::Attribute) -> syn::Result<Self> {
        if let Meta::Path(path) = &attr.meta {
            return Ok(Self {
                build: syn::LitStr::new("build", path.span()),
            });
        }
        if let Meta::NameValue(name_value) = &attr.meta {
            if name_value.path.is_ident("prototype") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) = &name_value.value
                {
                    return Ok(Self { build: lit.clone() });
                }
            }
        }
        Err(syn::Error::new(
            Span::call_site(),
            "invalid service definition, expected #[prototype] or #[prototype = \"build_func_name\"]",
        ))
    }
}

impl ToTokens for Service {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            ident,
            prototype,
            fields,
        } = self;
        let field_names: Vec<&syn::Ident> = fields.iter().map(|f| &f.field_name).collect();

        let output = match prototype {
            Some(Prototype { build }) => {
                let fn_name = syn::Ident::new(&build.value(), build.span());
                let (args, fields): (Vec<&Injectable>, Vec<&Injectable>) =
                    fields.iter().partition(|f| f.ty.is_arg());
                quote! {
                    impl #ident {
                        pub fn #fn_name(#(#args),*) -> ::spring::error::Result<Self> {
                            use ::spring::plugin::ComponentRegistry;
                            use ::spring::config::ConfigRegistry;
                            #(#fields)*
                            Ok(Self { #(#field_names),* })
                        }
                    }
                }
            }
            None => {
                let service_registrar =
                    syn::Ident::new(&format!("__ServiceRegistrarFor_{ident}"), ident.span());
                quote! {
                    impl ::spring::plugin::service::Service for #ident {
                        fn build<R>(app: &R) -> ::spring::error::Result<Self>
                        where
                            R: ::spring::plugin::ComponentRegistry + ::spring::config::ConfigRegistry
                        {
                            #(#fields)*
                            Ok(Self { #(#field_names),* })
                        }
                    }
                    #[allow(non_camel_case_types)]
                    struct #service_registrar;
                    impl ::spring::plugin::service::ServiceRegistrar for #service_registrar{
                        fn install_service(&self, app: &mut ::spring::app::AppBuilder)->::spring::error::Result<()> {
                            use ::spring::plugin::MutableComponentRegistry;
                            app.add_component(#ident::build(app)?);
                            Ok(())
                        }
                    }
                    ::spring::submit_service!(#service_registrar);
                }
            }
        };
        tokens.extend(output);
    }
}

pub(crate) fn expand_derive(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    Ok(Service::new(input)?.into_token_stream())
}
