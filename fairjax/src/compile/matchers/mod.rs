pub mod brute_force;
pub mod stateful_tree;

use crate::analyse::strategy::Strategy;
use crate::compile::case::{accept::AcceptCompiler, guard::GuardCompiler};
use crate::compile::matchers::brute_force::BruteForceCompiler;
use crate::compile::matchers::stateful_tree::{
    StatefulTreeCompiler, mappings::MappingCompiler, match_arms::MatchArmCompiler,
};
use crate::parse::context::Context;
use crate::traits::CaseBundle;
use proc_macro2::TokenStream;
use syn::Ident;

pub trait SetupCodeGen {
    fn generate(case: &dyn CaseBundle, context: Context, factory_ident: &Ident) -> TokenStream;
}

pub struct Setup;

impl SetupCodeGen for Setup {
    fn generate(case: &dyn CaseBundle, context: Context, factory_ident: &Ident) -> TokenStream {
        match case.strategy() {
            Strategy::StatefulTree => StatefulTreeCompiler::generate::<
                GuardCompiler,
                AcceptCompiler,
                MatchArmCompiler,
                MappingCompiler,
            >(case, context, factory_ident),
            Strategy::BruteForce => {
                BruteForceCompiler::generate::<GuardCompiler>(case, context, factory_ident)
            }
            Strategy::Partitions { vars, pattern } => todo!(),
        }
    }
}
