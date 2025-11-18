pub mod brute_force;
pub mod partitions;
pub mod stateful_tree;

use crate::analyse::{partition::Partitioning, strategy::Strategy};
use crate::compile::case::{accept::AcceptCompiler, guard::GuardCodeGen, guard::GuardCompiler};
use crate::compile::matchers::brute_force::BruteForceCompiler;
use crate::compile::matchers::partitions::PartitionsCompiler;
use crate::compile::matchers::stateful_tree::{
    StatefulTreeCompiler, mappings::MappingCompiler, match_arms::MatchArmCompiler,
};
use crate::compile::pattern::full::PatternCompiler;
use crate::parse::context::Context;
use crate::traits::CaseBundle;
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::Ident;

pub trait SetupCodeGen {
    fn generate(case: &dyn CaseBundle, context: Context, factory_ident: &Ident) -> TokenStream;
}

pub struct Setup;

impl SetupCodeGen for Setup {
    fn generate(bundle: &dyn CaseBundle, ctx: Context, factory_ident: &Ident) -> TokenStream {
        // Generate guard setup code
        let mut guard_fn_ident = bundle.case().ident_with_case_id("fairjax_guard_function");
        let guard_code =
            GuardCompiler::generate::<PatternCompiler>(bundle.case(), &ctx, &mut guard_fn_ident);

        // Generate code for partitioning middle-ware (if applicable)
        let (matcher_factory_ident, partitioning_code) =
            if let Some(Partitioning { vars, pattern }) = bundle.partitioning() {
                let matcher_factory_ident = bundle.case().ident_with_case_id("inner_matcher");
                let partitioning_code = PartitionsCompiler::generate(
                    &pattern,
                    &vars,
                    ctx.clone(),
                    bundle,
                    &factory_ident,
                    &matcher_factory_ident,
                );

                (matcher_factory_ident, partitioning_code)
            } else {
                (factory_ident.clone(), TokenStream::new())
            };

        // Generate backend matcher code
        let matcher_code = match bundle.strategy() {
            Strategy::StatefulTree => StatefulTreeCompiler::generate::<
                AcceptCompiler,
                MatchArmCompiler,
                MappingCompiler,
            >(
                bundle, ctx, &matcher_factory_ident, &guard_fn_ident
            ),
            Strategy::BruteForce => {
                BruteForceCompiler::generate(bundle, ctx, &matcher_factory_ident, &guard_fn_ident)
            }
        };

        // Assemble code snippets
        quote_spanned! { bundle.case().span() =>
            #guard_code
            #matcher_code
            #partitioning_code
        }
    }
}
