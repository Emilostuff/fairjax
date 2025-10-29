use crate::compile::{case::guard::GuardCodeGen, pattern::full::PatternCompiler};
use crate::parse::{case::Case, context::Context};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub struct BruteForceCompiler;

impl BruteForceCompiler {
    /// Generate declaration code for a BruteForceMatcher
    pub fn generate<G: GuardCodeGen>(
        case: &dyn Case,
        context: Context,
        output_ident: &Ident,
    ) -> TokenStream {
        // Generate guard
        let guard_ident = format_ident!("fairjax_bf_guard_function_{}", case.index());
        let guard_code = G::generate::<PatternCompiler>(case, &context, &guard_ident);

        // Retrieve values for code block
        let message_type = context.message_type;
        let pattern_size = case.pattern().len();

        // Assemble code snippets
        quote! {
            #guard_code

            let #output_ident = fairjax_core::strategies::brute_force::BruteForceMatcher::<#pattern_size, #message_type>::new(#guard_ident);
        }
    }
}
