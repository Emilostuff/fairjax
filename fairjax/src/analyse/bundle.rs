use crate::analyse::partition::Partitioning;
use crate::analyse::profile::PatternProfile;
use crate::parse::strategy::InputStrategy;
use crate::{analyse::strategy::Strategy, parse::case::CaseDefinition};
use proc_macro2::Span;
use syn::Result;

pub struct CaseBundleDefinition {
    pub case: CaseDefinition,
    pub partitioning: Option<Partitioning>,
    pub pattern_profile: PatternProfile,
    pub strategy: Strategy,
}

impl CaseBundleDefinition {
    pub fn analyse(mut case: CaseDefinition) -> Result<Self> {
        let partitioning = Partitioning::analyse(&mut case)?;

        let pattern_profile = PatternProfile::new(&case.pattern);

        if partitioning.is_some() && !pattern_profile.is_distinct() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Pattern must only have one occurrence per message variant when using partition variables.",
            ));
        }

        let strategy = match &case.strategy {
            InputStrategy::BruteForce => Strategy::BruteForce,
            InputStrategy::StatefulTree | InputStrategy::Auto => Strategy::StatefulTree,
        };

        Ok(Self {
            case,
            partitioning,
            pattern_profile,
            strategy,
        })
    }
}
