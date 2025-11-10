use crate::analyse::partition::Partitioning;
use crate::analyse::profile::PatternProfile;
use crate::parse::strategy::InputStrategy;
use crate::{analyse::strategy::Strategy, parse::case::CaseDefinition};
use proc_macro2::Span;
use syn::Result;

pub struct CaseBundleDefinition {
    pub case: CaseDefinition,
    pub pattern_profile: PatternProfile,
    pub strategy: Strategy,
}

impl CaseBundleDefinition {
    pub fn analyse(mut case: CaseDefinition) -> Result<Self> {
        let pattern_profile = PatternProfile::new(&case.pattern);
        let partitions = Partitioning::analyse(&case)?;

        use InputStrategy::*;
        let strategy = match partitions {
            Some(Partitioning { vars, clean_case }) => match case.strategy {
                BruteForce | StatefulTree => {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "Only Partitions strategy is compatible with partition variables.",
                    ));
                }
                Auto | Partitions => {
                    // Verify that pattern is distinct
                    if !pattern_profile.is_distinct() {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "Pattern must only have one occurrence per message variant when using partition variables.",
                        ));
                    }

                    // Swap case and inner case
                    let strategy = Strategy::Partitions {
                        vars,
                        pattern: case.pattern,
                    };
                    case = clean_case;
                    strategy
                }
            },
            None => match case.strategy {
                BruteForce => Strategy::BruteForce,
                StatefulTree | Auto => Strategy::StatefulTree,
                Partitions => {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "Partitions strategy chosen, but no partition variables found.",
                    ));
                }
            },
        };

        Ok(Self {
            case,
            pattern_profile,
            strategy,
        })
    }
}
