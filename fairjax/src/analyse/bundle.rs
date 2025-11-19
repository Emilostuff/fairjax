use crate::analyse::groups::SubPatternGroups;
use crate::analyse::partition::Partitioning;
use crate::parse::strategy::InputStrategy;
use crate::{analyse::strategy::Strategy, parse::case::CaseDefinition};
use proc_macro2::Span;
use syn::Result;

pub struct CaseBundleDefinition {
    pub case: CaseDefinition,
    pub partitioning: Option<Partitioning>,
    pub pattern_profile: SubPatternGroups,
    pub strategy: Strategy,
}

impl CaseBundleDefinition {
    pub fn analyse(mut case: CaseDefinition) -> Result<Self> {
        // Analysis
        let partitioning = Partitioning::analyse(&case)?;
        let groups = SubPatternGroups::new(&case.pattern);

        // Validate partitioning (if present)
        if let Some(partitioning) = &partitioning {
            if !groups.is_distinct() {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "Pattern must only have one occurrence per message variant when using partition variables.",
                ));
            } else {
                case = partitioning.cleaned_case.clone();
            }
        }

        // Choose the best, valid strategy
        let strategy = match &case.strategy {
            InputStrategy::BruteForce => Strategy::BruteForce,
            InputStrategy::StatefulTree | InputStrategy::Auto => Strategy::StatefulTree,
        };

        Ok(Self {
            case,
            partitioning,
            pattern_profile: groups,
            strategy,
        })
    }
}
