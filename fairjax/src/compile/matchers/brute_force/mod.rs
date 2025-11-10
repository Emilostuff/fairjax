use crate::compile::{case::guard::GuardCodeGen, pattern::full::PatternCompiler};
use crate::parse::context::Context;
use crate::traits::CaseBundle;
use proc_macro2::{Ident, TokenStream};
use quote::quote_spanned;

pub struct BruteForceCompiler;

impl BruteForceCompiler {
    /// Generate declaration code for a BruteForceMatcher
    pub fn generate<G: GuardCodeGen>(
        bundle: &dyn CaseBundle,
        context: Context,
        output_ident: &Ident,
    ) -> TokenStream {
        let case = bundle.case();

        // Generate guard
        let guard_fn_name = format!("fairjax_bf_guard_function_{}", case.index());
        let guard_fn_ident = Ident::new(&guard_fn_name, case.span());
        let guard_code = G::generate::<PatternCompiler>(case, &context, &guard_fn_name);

        // Retrieve values for code block
        let message_type = context.message_type;
        let pattern_size = case.pattern().len();

        // Assemble code snippets
        quote_spanned! {case.span() =>
            #guard_code

            let #output_ident = fairjax_core::strategies::brute_force::BruteForceMatcher::<#pattern_size, #message_type>::new(#guard_fn_ident);
        }
    }
}
