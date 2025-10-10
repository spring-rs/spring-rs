use crate::input_and_compile_error;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, LitStr};

pub(crate) fn on_connection(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = match syn::parse::<ItemFn>(input.clone()) {
        Ok(ast) => ast,
        Err(err) => return input_and_compile_error(input, err),
    };

    let handler_name = &ast.sig.ident;
    let handler_struct_name = syn::Ident::new(&format!("__SocketIOConnectHandler_{}", handler_name), handler_name.span());

    let output = quote! {
        #ast

        #[allow(non_camel_case_types)]
        pub struct #handler_struct_name;

        impl ::spring_web::handler::SocketIOHandlerRegistrar for #handler_struct_name {
            fn install_socketio_handlers(&self, socket: &::spring_web::socketioxide::extract::SocketRef) {
                use ::spring_web::socketioxide::handler::connect::ConnectHandler;
                use ::spring_web::socketioxide::adapter::LocalAdapter;
                use std::ops::Deref;
                
                // SocketRef is a newtype around Arc<Socket>, we need to extract it
                let socket_clone = socket.clone();
                // SocketRef derefs to Socket, so &*socket gives us &Socket
                // We need Arc<Socket>, so we clone the Arc through the SocketRef
                let socket_arc = unsafe {
                    // SocketRef is repr(transparent) over Arc<Socket>
                    std::mem::transmute::<::spring_web::socketioxide::extract::SocketRef, std::sync::Arc<::spring_web::socketioxide::socket::Socket<LocalAdapter>>>(socket_clone)
                };
                
                ::spring_web::socketioxide::handler::connect::ConnectHandler::call(&#handler_name, socket_arc, None);
            }
        }

        ::spring_web::handler::submit! {
            &(#handler_struct_name) as &dyn ::spring_web::handler::SocketIOHandlerRegistrar
        }
    };

    output.into()
}

pub(crate) fn on_disconnect(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = match syn::parse::<ItemFn>(input.clone()) {
        Ok(ast) => ast,
        Err(err) => return input_and_compile_error(input, err),
    };

    let handler_name = &ast.sig.ident;
    let handler_struct_name = syn::Ident::new(&format!("__SocketIODisconnectHandler_{}", handler_name), handler_name.span());

    let output = quote! {
        #ast

        #[allow(non_camel_case_types)]
        pub struct #handler_struct_name;

        impl ::spring_web::handler::SocketIOHandlerRegistrar for #handler_struct_name {
            fn install_socketio_handlers(&self, socket: &::spring_web::socketioxide::extract::SocketRef) {
                ::spring::tracing::info!("Installing on_disconnect handler: {}", stringify!(#handler_name));
                
                socket.on_disconnect(#handler_name);
            }
        }

        ::spring_web::handler::submit! {
            &(#handler_struct_name) as &dyn ::spring_web::handler::SocketIOHandlerRegistrar
        }
    };

    output.into()
}

pub(crate) fn subscribe_message(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = match syn::parse::<LitStr>(args) {
        Ok(args) => args,
        Err(err) => return input_and_compile_error(input, err),
    };

    let ast = match syn::parse::<ItemFn>(input.clone()) {
        Ok(ast) => ast,
        Err(err) => return input_and_compile_error(input, err),
    };

    let event_name = args.value();
    let handler_name = &ast.sig.ident;
    let handler_struct_name = syn::Ident::new(&format!("__SocketIOMessageHandler_{}_{}", event_name.replace("-", "_"), handler_name), handler_name.span());

    let has_ack_sender = ast.sig.inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Type::Path(type_path) = &*pat_type.ty {
                return type_path.path.segments.iter().any(|segment| 
                    segment.ident == "AckSender"
                );
            }
        }
        false
    });

    let output = if has_ack_sender {
        quote! {
            #ast

            #[allow(non_camel_case_types)]
            pub struct #handler_struct_name;

            impl ::spring_web::handler::SocketIOHandlerRegistrar for #handler_struct_name {
                fn install_socketio_handlers(&self, socket: &::spring_web::socketioxide::extract::SocketRef) {
                    use ::spring_web::socketioxide;
                    
                    socket.on(#event_name, async |data: socketioxide::extract::Data<::spring_web::rmpv::Value>, ack: socketioxide::extract::AckSender| {
                        #handler_name(data, ack).await
                    });
                }
            }

            ::spring_web::handler::submit! {
                &(#handler_struct_name) as &dyn ::spring_web::handler::SocketIOHandlerRegistrar
            }
        }
    } else {
        quote! {
            #ast

            #[allow(non_camel_case_types)]
            pub struct #handler_struct_name;

            impl ::spring_web::handler::SocketIOHandlerRegistrar for #handler_struct_name {
                fn install_socketio_handlers(&self, socket: &::spring_web::socketioxide::extract::SocketRef) {
                    use ::spring_web::socketioxide;
                    
                    socket.on(#event_name, async |socket: socketioxide::extract::SocketRef, data: socketioxide::extract::Data<::spring_web::rmpv::Value>| {
                        #handler_name(socket, data).await
                    });
                    
                }
            }

            ::spring_web::handler::submit! {
                &(#handler_struct_name) as &dyn ::spring_web::handler::SocketIOHandlerRegistrar
            }
        }
    };

    output.into()
}

pub(crate) fn on_fallback(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = match syn::parse::<ItemFn>(input.clone()) {
        Ok(ast) => ast,
        Err(err) => return input_and_compile_error(input, err),
    };

    let handler_name = &ast.sig.ident;
    let handler_struct_name = syn::Ident::new(&format!("__SocketIOFallbackHandler_{}", handler_name), handler_name.span());

    let output = quote! {
        #ast

        #[allow(non_camel_case_types)]
        pub struct #handler_struct_name;

        impl ::spring_web::handler::SocketIOHandlerRegistrar for #handler_struct_name {
            fn install_socketio_handlers(&self, socket: &::spring_web::socketioxide::extract::SocketRef) {
                socket.on_any(|socket: ::spring_web::socketioxide::extract::SocketRef, event: ::spring_web::socketioxide::extract::Event, data: ::spring_web::socketioxide::extract::Data<::spring_web::socketioxide::handler::Value>| async move {
                    #handler_name(socket, data).await;
                });
            }
        }

        ::spring_web::handler::submit! {
            &(#handler_struct_name) as &dyn ::spring_web::handler::SocketIOHandlerRegistrar
        }
    };

    output.into()
}