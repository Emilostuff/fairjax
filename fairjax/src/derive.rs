use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn expand_derive_message_trait(input: &DeriveInput) -> TokenStream {
    let name = input.ident.clone();

    quote! {
        impl fairjax_core::Message for #name {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_derive_message_trait() {
        let input: DeriveInput = parse_quote! {
            struct TestMessage {
                field: String,
            }
        };

        let output = expand_derive_message_trait(&input);
        let expected = quote! {
            impl fairjax_core::Message for TestMessage {}
        };

        assert_eq!(output.to_string(), expected.to_string());
    }
}
