use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, FnArg, Ident, ItemFn, Lit, MetaNameValue, Pat, PatIdent, Result, Token,
    Type,
};

pub fn test_me(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ParamArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let vis = &input_fn.vis;
    let asyncness = &input_fn.sig.asyncness;
    let output = &input_fn.sig.output;

    let user_block = &input_fn.block;

    // Extract the single function parameter
    let fn_args: Vec<_> = input_fn.sig.inputs.iter().collect();
    if fn_args.len() != 1 {
        return syn::Error::new_spanned(
            &input_fn.sig.inputs,
            "Function must take exactly one argument: `param_one: String`",
        )
        .to_compile_error()
        .into();
    }

    let param_name = if let FnArg::Typed(pat_type) = &fn_args[0] {
        if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
            ident.clone()
        } else {
            return syn::Error::new_spanned(&pat_type.pat, "Expected identifier")
                .to_compile_error()
                .into();
        }
    } else {
        return syn::Error::new_spanned(fn_args[0], "Unsupported function signature")
            .to_compile_error()
            .into();
    };

    let param_one_value = args
        .param_one
        .unwrap_or_else(|| "DEFAULT_VALUE".to_string());

    let expanded = quote! {
        #[tokio::test]
        #vis #asyncness fn #fn_name() #output {
            let #param_name = #param_one_value.to_string();
            #user_block
        }
    };

    expanded.into()
}
