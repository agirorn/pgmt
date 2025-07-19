use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, FnArg, ItemFn, Lit, MetaNameValue, Pat, PatIdent, Result, Token, Type,
};

struct PgmtArgs {
    migrations: Option<Vec<String>>,
    placeholders: std::collections::HashMap<String, String>,
}

impl Parse for PgmtArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut migrations = Vec::new();
        let mut placeholders: HashMap<String, String> = HashMap::new();

        while !input.is_empty() {
            let name_value: MetaNameValue = input.parse()?;
            let ident = name_value.path.get_ident().ok_or_else(|| {
                syn::Error::new_spanned(
                    &name_value.path,
                    "Expected ident like `param_one` or `param_two`",
                )
            })?;

            match ident.to_string().as_str() {
                "migrations" => {
                    if let syn::Expr::Lit(expr_lit) = &name_value.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            migrations.push(lit_str.value());
                        } else {
                            return Err(syn::Error::new_spanned(
                                &expr_lit.lit,
                                "Expected string literal for `migrations`",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            &name_value.value,
                            "Expected literal expression for `migrations`",
                        ));
                    }
                }
                s if s.starts_with("placeholder_") => {
                    let key = ident
                        .to_string()
                        .trim_start_matches("placeholder_")
                        .to_string();

                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &name_value.value
                    {
                        placeholders.insert(key, lit_str.value());
                    } else {
                        return Err(syn::Error::new_spanned(
                            name_value.value.clone(),
                            "Expected string literal",
                        ));
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(name_value, "Unsupported argument"));
                }
            }

            // Globber up the, commans between args
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let migrations = if migrations.is_empty() {
            None
        } else {
            Some(migrations)
        };

        Ok(PgmtArgs {
            migrations,
            placeholders,
        })
    }
}

pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as PgmtArgs);
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let vis = &input_fn.vis;
    let asyncness = &input_fn.sig.asyncness;
    let output = &input_fn.sig.output;

    let user_block = &input_fn.block;

    let pool_arg = if input_fn.sig.inputs.len() != 1 {
        return syn::Error::new_spanned(
            &input_fn.sig.inputs,
            "Expected exactly one argument: `pool: pgmt::Pool`",
        )
        .to_compile_error()
        .into();
    } else if let Some(FnArg::Typed(arg)) = input_fn.sig.inputs.first() {
        match (&*arg.pat, &*arg.ty) {
            (Pat::Ident(PatIdent { ident, .. }), Type::Path(ty)) => {
                let arg_name = ident;
                let ty_path = &ty.path;
                quote! { #arg_name: #ty_path }
            }
            _ => {
                return syn::Error::new_spanned(
                    arg,
                    "Expected argument of the form `pool: pgmt::Pool`",
                )
                .to_compile_error()
                .into();
            }
        }
    } else {
        return syn::Error::new_spanned(&input_fn.sig.inputs, "Unsupported function signature")
            .to_compile_error()
            .into();
    };

    let arg_name = if let FnArg::Typed(arg) = input_fn.sig.inputs.first().unwrap() {
        if let Pat::Ident(pat_ident) = &*arg.pat {
            &pat_ident.ident
        } else {
            panic!("Expected ident pattern");
        }
    } else {
        panic!("Expected typed argument");
    };

    let default_migrations: Vec<String> = vec![];
    let migrations = args.migrations.unwrap_or(default_migrations);
    let migrations = migrations.iter().map(|s| quote! { #s.to_string() });
    let placeholders = args.placeholders;
    let placeholders = placeholders
        .iter()
        .map(|(k, v)| quote! { (#k.to_string(), #v.to_string()) });

    let expanded = quote! {
        #[tokio::test]
        #vis #asyncness fn #fn_name() #output {
            let migrations: Vec<String> = vec![#(#migrations),*];
            let placeholders = Some(std::collections::HashMap::from([#(#placeholders),*]));

            async fn inner(#pool_arg) #output {
                #user_block
            }

            pgmt_core::test_migration(migrations, placeholders, async move |#arg_name| {
                inner(#arg_name).await;
            }).await;
        }
    };

    expanded.into()
}
