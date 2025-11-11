use crate::parse::context::Context;
use crate::traits::CaseBundle;
use proc_macro2::{Ident, TokenStream};
use quote::quote_spanned;

pub struct BruteForceCompiler;

impl BruteForceCompiler {
    /// Generate declaration code for a BruteForceMatcher
    pub fn generate(
        bundle: &dyn CaseBundle,
        context: Context,
        factory_ident: &Ident,
        guard_fn_ident: &Ident,
    ) -> TokenStream {
        let case = bundle.case();

        // Retrieve values for code block
        let message_type = context.message_type;
        let pattern_size = case.pattern().len();

        // Assemble code snippets
        quote_spanned! {case.span() =>
            let #factory_ident = || {
                Box::new(
                    fairjax_core::strategies::brute_force::BruteForceMatcher::<
                        #pattern_size,
                        #message_type
                    >::new(#guard_fn_ident)
                ) as Box<dyn fairjax_core::CaseHandler<#message_type>>
            };
        }
    }
}
