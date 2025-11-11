pub mod brute_force;
pub mod partitions;
pub mod stateful_tree;

use crate::analyse::strategy::Strategy;
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
    fn generate(bundle: &dyn CaseBundle, context: Context, factory_ident: &Ident) -> TokenStream {
        // Down stream dependencies
        let stateful =
            StatefulTreeCompiler::generate::<AcceptCompiler, MatchArmCompiler, MappingCompiler>;
        let bruteforce = BruteForceCompiler::generate;
        let partitions = PartitionsCompiler::generate;
        let guard = GuardCompiler::generate::<PatternCompiler>;

        // Generate guard setup code
        let guard_fn_ident = Ident::new(
            &format!("fairjax_guard_function{}", bundle.case().index()),
            bundle.case().span(),
        );
        let guard_code = guard(bundle.case(), &context, guard_fn_ident.clone());

        // Generate matcher setup code
        let matcher_code = match bundle.strategy() {
            Strategy::StatefulTree => stateful(bundle, context, factory_ident, &guard_fn_ident),
            Strategy::BruteForce => bruteforce(bundle, context, factory_ident, &guard_fn_ident),
            Strategy::Partitions { vars, pattern } => {
                let inner_factory_ident = Ident::new(
                    &format!("inner_matcher{}", bundle.case().index()),
                    bundle.case().span(),
                );
                let stateful_tree_code = stateful(
                    bundle,
                    context.clone(),
                    &inner_factory_ident,
                    &guard_fn_ident,
                );
                let partitions_code = partitions(
                    pattern,
                    vars,
                    context,
                    bundle,
                    &factory_ident,
                    &inner_factory_ident,
                );
                quote_spanned! { bundle.case().span() =>
                    #stateful_tree_code
                    #partitions_code
                }
            }
        };

        quote_spanned! { bundle.case().span() =>
            #guard_code
            #matcher_code
        }
    }
}
