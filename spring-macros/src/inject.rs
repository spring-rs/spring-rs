use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    AngleBracketedGenericArguments, GenericArgument, Meta, MetaList, PathArguments, Token, Type,
    TypePath,
};

fn inject_error_tip() -> syn::Error {
    syn::Error::new(
        Span::call_site(),
        "inject Service only support Named-field Struct",
    )
}

enum InjectableType {
    Component(syn::Path),
    Config(syn::Path),
    ComponentRef(syn::Path),
    ConfigRef(syn::Path),
    MethodCall(syn::ExprMethodCall),
}

enum InjectableAttr {
    Component,
    Config,
    MethodCall(syn::ExprMethodCall),
}

struct Injectable {
    ty: InjectableType,
    field_name: syn::Ident,
}

impl Injectable {
    fn new(field: syn::Field) -> syn::Result<Self> {
        let ty = Self::compute_type(&field)?;
        let field_name = field.ident.ok_or_else(inject_error_tip)?;
        Ok(Self { ty, field_name })
    }

    fn compute_type(field: &syn::Field) -> syn::Result<InjectableType> {
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
                return Ok(attr.to_type(ty));
            } else {
                Err(syn::Error::new_spanned(
                    inject_attr,
                    "invalid inject definition, expected #[inject(component|config|method(args))]",
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
        let field_name = &field
            .ident
            .clone()
            .map(|ident| ident.to_string())
            .ok_or_else(inject_error_tip)?;
        Err(syn::Error::new_spanned(
            &field,
            format!(
                "{field_name} field missing inject definition, expected #[inject(component|config|method(args))]",
            ),
        ))
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
        if name.is_ident("method") {
            input.parse::<Token![=]>()?;
            let method_call = input.parse::<syn::ExprMethodCall>()?;
            return Ok(Self::MethodCall(method_call));
        }
        Err(syn::Error::new(
            Span::call_site(),
            "invalid inject definition, expected #[inject(component|config|method(args))]",
        ))
    }
}

impl InjectableAttr {
    fn to_type(self, ty: &syn::Path) -> InjectableType {
        match self {
            Self::Component => InjectableType::Component(ty.clone()),
            Self::Config => InjectableType::Config(ty.clone()),
            Self::MethodCall(method_call) => InjectableType::MethodCall(method_call),
        }
    }
}

impl ToTokens for Injectable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { ty, field_name } = self;
        match ty {
            InjectableType::Component(type_path) => {
                tokens.extend(quote! {
                    let #field_name = app.try_get_component::<#type_path>()?;
                });
            }
            InjectableType::Config(type_path) => {
                tokens.extend(quote! {
                    let #field_name = app.get_config::<#type_path>()?;
                });
            }
            InjectableType::ComponentRef(type_path) => {
                tokens.extend(quote! {
                    let #field_name = app.try_get_component_ref::<#type_path>()?;
                });
            }
            InjectableType::ConfigRef(type_path) => {
                tokens.extend(quote! {
                    let #field_name = ::spring::config::ConfigRef::new(app.get_config::<#type_path>()?);
                });
            }
            InjectableType::MethodCall(method_call) => {
                tokens.extend(quote! {
                    let #field_name = #method_call;
                });
            }
        }
    }
}

struct Service {
    fields: Vec<Injectable>,
}

impl Service {
    fn new(fields: syn::Fields) -> syn::Result<Self> {
        let fields = fields
            .into_iter()
            .map(Injectable::new)
            .collect::<syn::Result<Vec<_>>>()?;
        Ok(Self { fields })
    }
}

impl ToTokens for Service {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field_names: Vec<&syn::Ident> = self.fields.iter().map(|f| &f.field_name).collect();
        let fields = &self.fields;
        tokens.extend(quote! {
            #(#fields)*
            Ok(Self { #(#field_names),* })
        });
    }
}

pub(crate) fn expand_derive(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let service = if let syn::Data::Struct(data) = input.data {
        Service::new(data.fields)?
    } else {
        return Err(inject_error_tip());
    };
    let ident = input.ident;
    let service_registrar =
        syn::Ident::new(&format!("__ServiceRegistrarFor_{ident}"), ident.span());

    let output = quote! {
        impl ::spring::plugin::service::Service for #ident {
            fn build<R>(app: &R) -> ::spring::error::Result<Self>
            where
                R: ::spring::plugin::ComponentRegistry + ::spring::config::ConfigRegistry
            {
                #service
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
    };

    Ok(output)
}
