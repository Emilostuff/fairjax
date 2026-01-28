pub mod clean;
pub mod counts;
pub mod var;

use crate::analyse::partition::{clean::SubPatternCleaner, counts::IdentCounts, var::UnitingVars};
use crate::parse::pattern::PatternDefinition;
use crate::traits::Pattern;
use syn::Result;

pub struct Partitioning {
    pub vars: Vec<String>,
    pub original_pattern: PatternDefinition,
    pub cleaned_pattern: PatternDefinition,
}

impl Partitioning {
    pub fn analyse(pattern: &PatternDefinition) -> Result<Option<Self>> {
        // If pattern contains a single message, partitioning will be irrelevant
        if pattern.len() < 2 {
            return Ok(None);
        }

        // Tally up identifier occurrences in pattern
        let counts = IdentCounts::analyse(&pattern)?;

        // Handle result
        match UnitingVars::identify(counts, pattern.len())? {
            vars if !vars.is_empty() => Ok(Some(Self {
                vars: vars.clone(),
                original_pattern: pattern.clone(),
                cleaned_pattern: Self::get_cleaned_pattern(&pattern, &vars),
            })),
            _ => Ok(None),
        }
    }

    /// Clean sub-patterns, but skip first sub-pattern to keep idents in scope for body and guard code
    fn get_cleaned_pattern(
        pattern: &PatternDefinition,
        uniting_vars: &Vec<String>,
    ) -> PatternDefinition {
        let mut cleaned_pattern = pattern.clone();
        cleaned_pattern
            .sub_patterns
            .split_at_mut(1)
            .1
            .iter_mut()
            .for_each(|sp| SubPatternCleaner::clean(sp, uniting_vars));
        cleaned_pattern
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::SubPattern;

    use super::*;
    use proc_macro_utils::assert_tokens;
    use quote::ToTokens;
    use syn::{Pat, parse_quote};

    #[test]
    fn test_clean_pattern_preserves_order() {
        // Define inputs
        let input: Pat = parse_quote!((A(x), B(x), C(x), D(x)));
        let pattern = PatternDefinition::parse(input).unwrap();

        // get cleaned pattern
        let result = Partitioning::get_cleaned_pattern(&pattern, &vec![]);

        // Extract and verify results
        let result_idents = result
            .sub_patterns
            .iter()
            .map(|sp| sp.get_identifier().to_string())
            .collect::<Vec<_>>();

        assert_eq!(vec!["A", "B", "C", "D"], result_idents);
    }

    #[test]
    fn test_uniting_var_analyse_simple() {
        // Define inputs
        let input: Pat = parse_quote!((A(x), B(x)));

        let pattern = PatternDefinition::parse(input).unwrap();

        // Analyze input
        let result = Partitioning::analyse(&pattern).unwrap().unwrap();

        // Extract and verify results
        let sub_patterns = result
            .cleaned_pattern
            .sub_patterns
            .iter()
            .map(|sp| sp.to_syn_pattern())
            .collect::<Vec<_>>();

        // Verify correct uniting variable is extracted
        assert_eq!(vec!["x"], result.vars);

        // Verify pattern is cleaned correctly
        assert_tokens!(sub_patterns[0].to_token_stream(), { A(x) });
        assert_tokens!(sub_patterns[1].to_token_stream(), { B(_) });
    }

    #[test]
    fn test_uniting_var_analyse_nested() {
        // Define inputs
        let input: Pat = parse_quote!((A { x: (_, id), .. }, B(_, id), C { id }));
        let pattern = PatternDefinition::parse(input).unwrap();

        // Analyze input
        let result = Partitioning::analyse(&pattern).unwrap().unwrap();

        // Verify results
        assert_eq!(vec!["id"], result.vars);
    }

    #[test]
    fn test_uniting_var_analyse_none() {
        // Define inputs
        let input: Pat = parse_quote!((A { x: (_, id), .. }, B(_, id2), C { id3 }));
        let pattern = PatternDefinition::parse(input).unwrap();

        // Parse input and ensure no uniting vars are found
        assert!(Partitioning::analyse(&pattern).unwrap().is_none());
    }

    #[test]
    fn test_uniting_var_analyse_multiple_vars() {
        // Define inputs
        let input: Pat = parse_quote!((A { x: (_, id), data }, B(id, data), C { id, data }));
        let pattern = PatternDefinition::parse(input).unwrap();

        // Analyze input
        let result = Partitioning::analyse(&pattern).unwrap().unwrap();

        // Extract and verify results
        let sub_patterns = result
            .cleaned_pattern
            .sub_patterns
            .iter()
            .map(|sp| sp.to_syn_pattern())
            .collect::<Vec<_>>();

        // Verify correct uniting variable is extracted
        assert_eq!(vec!["data", "id"], result.vars);

        // Verify pattern is cleaned correctly
        #[rustfmt::skip]
        assert_tokens!(sub_patterns[0].to_token_stream(), { A { x: (_, id), data } });
        assert_tokens!(sub_patterns[1].to_token_stream(), { B(_, _) });
        assert_tokens!(sub_patterns[2].to_token_stream(), { C { .. } });
    }

    #[test]
    #[should_panic]
    fn test_uniting_var_analyse_failing() {
        // Define inputs
        let input: Pat = parse_quote!((A { x: (_, id), .. }, B(id, id), C { id }));
        let pattern = PatternDefinition::parse(input).unwrap();

        // Try to analyse
        Partitioning::analyse(&pattern).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_uniting_var_analyse_failing_incomplete() {
        // Define inputs
        let input: Pat = parse_quote! {
            (A { x: (_, toast), data }, B(id, _), C { id } )
        };
        let pattern = PatternDefinition::parse(input).unwrap();

        // Try to analyse
        Partitioning::analyse(&pattern).unwrap();
    }
}
