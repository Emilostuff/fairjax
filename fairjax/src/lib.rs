extern crate proc_macro;
use proc_macro::TokenStream;

use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Message)]
pub fn my_trait_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl fairjax_core::Message for #name {}
    };

    TokenStream::from(expanded)
}
