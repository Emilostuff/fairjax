pub mod clean;
pub mod counts;
pub mod var;

use crate::analyse::partition::{
    clean::SubPatternCleaner, counts::IdentCounts, var::PartitionVars,
};
use crate::parse::case::CaseDefinition;
use crate::parse::pattern::PatternDefinition;
use crate::traits::Pattern;
use syn::Result;

pub struct Partitioning {
    pub vars: Vec<String>,
    pub pattern: PatternDefinition,
    pub cleaned_case: CaseDefinition,
}

impl Partitioning {
    pub fn analyse(case: &CaseDefinition) -> Result<Option<Self>> {
        let mut case = case.clone();

        // If pattern contains a single message, partitioning will be irrelevant
        if case.pattern.len() < 2 {
            return Ok(None);
        }

        // Tally up identifier occurrences in pattern
        let counts = IdentCounts::analyse(&case.pattern)?;

        // Handle result
        match PartitionVars::identify(counts, case.pattern.len())? {
            vars if !vars.is_empty() => {
                // Extract original pattern with partition vars
                let pattern = case.pattern.clone();

                // Clean case from partition vars (except first sub-pattern)
                Self::clean_case(&mut case, &vars);

                Ok(Some(Self {
                    vars: vars.clone(),
                    pattern,
                    cleaned_case: case,
                }))
            }
            _ => Ok(None),
        }
    }

    fn clean_case(case: &mut CaseDefinition, partition_vars: &Vec<String>) {
        // Clean sub-patterns, but skip first sub-pattern to keep idents in scope for body and guard code
        case.pattern
            .sub_patterns
            .split_at_mut(1)
            .1
            .iter_mut()
            .for_each(|sp| SubPatternCleaner::clean(sp, partition_vars));
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::SubPattern;

    use super::*;
    use proc_macro_utils::assert_tokens;
    use quote::ToTokens;
    use syn::{Arm, parse_quote};

    #[test]
    fn test_clean_case_preserves_order() {
        // Define inputs
        let input: Arm = parse_quote! {
            (A(x), B(x), C(x), D(x)) => ()
        };
        let mut case = CaseDefinition::parse(input, 0).unwrap();

        // Run clean case
        Partitioning::clean_case(&mut case, &vec![]);

        // Extract and verify results
        let result_idents = case
            .pattern
            .sub_patterns
            .iter()
            .map(|sp| sp.get_identifier().to_string())
            .collect::<Vec<_>>();

        assert_eq!(vec!["A", "B", "C", "D"], result_idents);
    }

    #[test]
    fn test_partition_var_analyse_simple() {
        // Define inputs
        let input: Arm = parse_quote! {
            (A(x), B(x)) => ()
        };

        let case = CaseDefinition::parse(input, 0).unwrap();

        // Analyze input
        let result = Partitioning::analyse(&case).unwrap().unwrap();

        // Extract and verify results
        let sub_patterns = result
            .cleaned_case
            .pattern
            .sub_patterns
            .iter()
            .map(|sp| sp.to_pattern())
            .collect::<Vec<_>>();

        // Verify correct partition variable is extracted
        assert_eq!(vec!["x"], result.vars);

        // Verify pattern is cleaned correctly
        assert_tokens!(sub_patterns[0].to_token_stream(), { A(x) });
        assert_tokens!(sub_patterns[1].to_token_stream(), { B(_) });
    }

    #[test]
    fn test_partition_var_analyse_nested() {
        // Define inputs
        let input: Arm = parse_quote! {
            (A { x: (_, id), .. }, B(_, id), C { id } ) => ()
        };
        let case = CaseDefinition::parse(input, 0).unwrap();

        // Analyze input
        let result = Partitioning::analyse(&case).unwrap().unwrap();

        // Verify results
        assert_eq!(vec!["id"], result.vars);
    }

    #[test]
    fn test_partition_var_analyse_none() {
        // Define inputs
        let input: Arm = parse_quote! {
            (A { x: (_, id), .. }, B(_, id2), C { id3 } ) => ()
        };
        let case = CaseDefinition::parse(input, 0).unwrap();

        // Parse input and ensure no partition vars are found
        assert!(Partitioning::analyse(&case).unwrap().is_none());
    }

    #[test]
    fn test_partition_var_analyse_multiple_vars() {
        // Define inputs
        let input: Arm = parse_quote! {
            (A { x: (_, id), data }, B(id, data), C { id, data } ) => ()
        };
        let case = CaseDefinition::parse(input, 0).unwrap();

        // Analyze input
        let result = Partitioning::analyse(&case).unwrap().unwrap();

        // Extract and verify results
        let sub_patterns = result
            .cleaned_case
            .pattern
            .sub_patterns
            .iter()
            .map(|sp| sp.to_pattern())
            .collect::<Vec<_>>();

        // Verify correct partition variable is extracted
        assert_eq!(vec!["data", "id"], result.vars);

        // Verify pattern is cleaned correctly
        #[rustfmt::skip]
        assert_tokens!(sub_patterns[0].to_token_stream(), { A { x: (_, id), data } });
        assert_tokens!(sub_patterns[1].to_token_stream(), { B(_, _) });
        assert_tokens!(sub_patterns[2].to_token_stream(), { C { .. } });
    }

    #[test]
    #[should_panic]
    fn test_partition_var_analyse_failing() {
        // Define inputs
        let input: Arm = parse_quote! {
            (A { x: (_, id), .. }, B(id, id), C { id } ) => ()
        };
        let case = CaseDefinition::parse(input, 0).unwrap();

        // Try to analyse
        Partitioning::analyse(&case).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_partition_var_analyse_failing_incomplete() {
        // Define inputs
        let input: Arm = parse_quote! {
            (A { x: (_, toast), data }, B(id, _), C { id } ) => ()
        };
        let case = CaseDefinition::parse(input, 0).unwrap();

        // Try to analyse
        Partitioning::analyse(&case).unwrap();
    }
}
