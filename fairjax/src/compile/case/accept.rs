use crate::compile::pattern::sub::SubPatternCodeGen;
use crate::parse::context::Context;
use crate::traits::Case;
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::Ident;

pub trait AcceptCodeGen {
    fn generate<'a, S: SubPatternCodeGen>(
        case: &dyn Case,
        context: &Context,
        fn_name: &'a str,
    ) -> TokenStream;
}

pub struct AcceptCompiler;

impl AcceptCodeGen for AcceptCompiler {
    fn generate<'a, S: SubPatternCodeGen>(
        case: &dyn Case,
        context: &Context,
        fn_name: &'a str,
    ) -> TokenStream {
        let span = case.span();

        // Construct guard function identifier
        let fn_ident = Ident::new(fn_name, span);

        // Retrieve values for code block
        let message_type = &context.message_type;

        let sub_patterns: Vec<_> = case
            .pattern()
            .sub_patterns()
            .iter()
            .map(|sp| S::generate(*sp, false))
            .collect();

        quote_spanned! {
            span =>
            fn #fn_ident(input: &#message_type,) -> bool {
                #[allow(unreachable_patterns, unused_variables)]
                match input {
                    #(#sub_patterns => true), *,
                    _ => false,
                }
            }
        }
    }
}
