#![crate_type = "proc-macro"]

extern crate proc_macro;
mod macros;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::test::test(attr, item)
}
