pub mod brute_force;
pub mod stateful_tree;

use crate::compile::case::accept::AcceptCompiler;
use crate::compile::case::guard::GuardCompiler;
use crate::compile::matchers::brute_force::BruteForceCompiler;
use crate::compile::matchers::stateful_tree::StatefulTreeCompiler;
use crate::compile::matchers::stateful_tree::element_mapping::ElementMappingCompiler;
use crate::compile::matchers::stateful_tree::match_arms::MatchArmCompiler;
use crate::parse::{case::Case, context::Context, strategy::Strategy};
use proc_macro2::TokenStream;
use syn::Ident;

pub trait SetupCodeGen {
    fn generate(case: &dyn Case, context: Context, output_ident: &Ident) -> TokenStream;
}

pub struct Setup;

impl SetupCodeGen for Setup {
    fn generate(case: &dyn Case, context: Context, output_ident: &Ident) -> TokenStream {
        match case.strategy() {
            Strategy::Auto | Strategy::StatefulTree => StatefulTreeCompiler::generate::<
                GuardCompiler,
                AcceptCompiler,
                MatchArmCompiler,
                ElementMappingCompiler,
            >(case, context, output_ident),
            Strategy::BruteForce => {
                BruteForceCompiler::generate::<GuardCompiler>(case, context, output_ident)
            }
        }
    }
}
