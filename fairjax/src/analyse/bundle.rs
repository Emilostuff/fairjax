use crate::analyse::content::SubPatternContents;
use crate::analyse::groups::SubPatternGroups;
use crate::analyse::partition::Partitioning;
use crate::parse::strategy::InputStrategy;
use crate::{analyse::strategy::Strategy, parse::case::CaseDefinition};
use proc_macro2::Span;
use syn::Result;

pub struct CaseBundleDefinition {
    pub case: CaseDefinition,
    pub partitioning: Option<Partitioning>,
    pub groups: SubPatternGroups,
    pub strategy: Strategy,
}

impl CaseBundleDefinition {
    pub fn analyse(mut case: CaseDefinition) -> Result<Self> {
        // Analysis
        let partitioning = Partitioning::analyse(&case.pattern)?;
        let groups = SubPatternGroups::analyse(&case.pattern);
        let contents = SubPatternContents::extract(&case.pattern);

        // Validate partitioning (if present)
        if let Some(partitioning) = &partitioning {
            for group in &groups.0 {
                if group.size() > 1 {
                    let (first, rest) = group.occurrences().split_at(1);
                    let first = &contents[first[0]];
                    for other in rest {
                        let other = &contents[*other];

                        for uniting_var in &partitioning.vars {
                            if !SubPatternContents::same_placements(&uniting_var, first, other) {
                                return Err(syn::Error::new(
                                    Span::call_site(),
                                    "Pattern must only have one occurrence per message variant when using uniting variables.",
                                ));
                            }
                        }
                    }
                }
            }
            // Partitioning is possible
            case.pattern = partitioning.cleaned_pattern.clone();
        }

        // Choose the best, valid strategy
        let strategy = match &case.strategy {
            InputStrategy::BruteForce => Strategy::BruteForce,
            InputStrategy::StatefulTree | InputStrategy::Auto => Strategy::StatefulTree,
        };

        Ok(Self {
            case,
            partitioning,
            groups,
            strategy,
        })
    }
}
