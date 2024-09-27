use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Attribute, Field, Type};

enum InjectableType {
    Component,
    Config,
}

struct Injectable {
    ty: InjectableType,
    field_name: syn::Ident,
    type_path: syn::Path,
}

impl Injectable {
    fn new(field: syn::Field) -> syn::Result<Self> {
        let syn::Field {
            ident, ty, attrs, ..
        } = field;
        let type_path = if let syn::Type::Path(path) = ty {
            path.path
        } else {
            Err(syn::Error::new(
                Span::call_site(),
                "inject Service only support Named-field Struct",
            ))?
        };
        Ok(Self {
            ty: Self::compute_type(attrs, type_path.clone()),
            field_name: ident.ok_or_else(|| {
                syn::Error::new(
                    Span::call_site(),
                    "inject Service only support Named-field Struct",
                )
            })?,
            type_path,
        })
    }

    fn compute_type(attrs: Vec<Attribute>, ty: syn::Path) -> InjectableType {
        for attr in attrs {
            if attr.path().is_ident("config") {
                return InjectableType::Config;
            }
            if attr.path().is_ident("component") {
                return InjectableType::Component;
            }
        }

        InjectableType::Component
    }
}

impl ToTokens for Injectable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            ty,
            field_name,
            type_path,
        } = self;
        match ty {
            InjectableType::Component => {
                tokens.extend(quote! {
                    #field_name: app.get_component::<#type_path>()
                });
            }
            InjectableType::Config => {
                tokens.extend(quote! {
                    #field_name: app.get_config::<#type_path>()?
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
        let fields = &self.fields;
        tokens.extend(quote! {
            Self {
                #(#fields),*
            }
        });
    }
}

pub(crate) fn expand_derive(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let service = if let syn::Data::Struct(data) = input.data {
        Service::new(data.fields)?
    } else {
        return Err(syn::Error::new(
            Span::call_site(),
            "inject Service only support Named-field Struct",
        ));
    };
    let ident = input.ident;

    let output = quote! {
        impl ::spring::plugin::service::Service for #ident {
            fn build(app: &::spring::app::AppBuilder) -> ::spring::error::Result<Self> {
                use ::spring::config::ConfigRegistry;
                Ok(#service)
            }
        }
    };

    Ok(output)
}
