use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::Expr;
use syn::ExprCall;
use syn::ExprMethodCall;
use syn::{ItemFn, Stmt, Token};

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
    fn new(args: ConfigArgs, ast: ItemFn) -> Self {
        Self { args, ast }
    }
}

impl ToTokens for AppConfig {
    fn to_tokens(&self, output: &mut proc_macro2::TokenStream) {
        let mut input_fn = self.ast.clone();

        // 遍历语句，寻找包含 `App::new()` 的语句
        // for stmt in &mut input_fn.block.stmts {
        //     if let Stmt::Expr(
        //         Expr::Call(ExprCall {
        //             attrs,
        //             func,
        //             paren_token,
        //             args,
        //         }),
        //         _,
        //     ) = stmt
        //     {
        //         eprintln!("{:?}", stmt);
        //         if let Expr::Path(path) = &**func {
        //             // 确保我们匹配 `App::new()` 调用
        //             if path.path.is_ident("new")
        //                 && path
        //                     .path
        //                     .segments
        //                     .first()
        //                     .map_or(false, |seg| seg.ident == "App")
        //             {
        //                 let add_router_call: ExprMethodCall = syn::parse_quote! {
        //                     .add_router(::spring_web::handler::auto_router())
        //                 };
        //                 // 构造新的调用链，保留原有的属性
        //                 let new_expr = Expr::MethodCall(ExprMethodCall {
        //                     attrs: Vec::new(), // 没有附加属性，因此使用空向量
        //                     receiver: Box::new(Expr::Call(ExprCall {
        //                         func: func.clone(),
        //                         args: args.clone(),
        //                         attrs: attrs.clone(), // 保留原有的属性
        //                         paren_token: paren_token.clone(),
        //                     })),
        //                     method: add_router_call.method.clone(),
        //                     turbofish: None, // 没有 turbofish 运算符
        //                     args: add_router_call.args.clone(),
        //                     paren_token: add_router_call.paren_token,
        //                     dot_token: add_router_call.dot_token,
        //                 });

        //                 // 将新的表达式替换原有的表达式
        //                 *stmt = Stmt::Expr(new_expr, None); // `None` 表示没有尾随分号
        //             }
        //         }
        //     }
        // }

        // 生成输出的TokenStream
        output.extend(quote! {
            #input_fn
        });
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

    AppConfig::new(args, ast).into_token_stream().into()
}
