use crate::compile::pattern::sub::SubPatternCodeGen;
use crate::traits::CaseBundle;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;

pub trait MatchArmCodeGen {
    fn generate<S: SubPatternCodeGen>(span: Span, bundle: &dyn CaseBundle) -> TokenStream;
}

pub struct MatchArmCompiler;

impl MatchArmCodeGen for MatchArmCompiler {
    fn generate<S: SubPatternCodeGen>(span: Span, bundle: &dyn CaseBundle) -> TokenStream {
        let match_arms = bundle
            .sub_pattern_groups()
            .0
            .iter()
            .scan(0, |position, stats| {
                // Create anonymized sub pattern
                let anonymized_sub_pattern =
                    S::generate(bundle.sub_pattern_at_index(stats.first()), true);

                // Determine which positions in the data structure should be reserved
                // for this sub-pattern type
                let start = *position;
                let end = start + stats.size();

                // Update folding iterator
                *position = end;

                // Assemble code snippets into match arm
                Some(quote_spanned!(span => #anonymized_sub_pattern => (#start, #end)))
            });

        // Combine, and comma separate match arms
        quote_spanned!(span => #(#match_arms),*,)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyse::groups::{Group, SubPatternGroups};
    use crate::analyse::partition::Partitioning;
    use crate::analyse::strategy::Strategy;
    use crate::compile::pattern::sub::SubPatternCodeGen;
    use crate::parse::sub_pattern::SubPatternDefinition;
    use crate::traits::{Case, SubPattern};
    use proc_macro_utils::assert_tokens;
    use proc_macro2::{Ident, TokenStream};
    use quote::ToTokens;

    fn ident(input: &str) -> Ident {
        Ident::new(input, proc_macro2::Span::call_site())
    }

    struct MockCaseBundle {
        sub_patterns: Vec<MockSubPattern>,
        profile: SubPatternGroups,
    }

    impl CaseBundle for MockCaseBundle {
        fn case(&self) -> &dyn Case {
            unimplemented!()
        }
        fn strategy(&self) -> &Strategy {
            unimplemented!()
        }

        fn sub_pattern_groups(&self) -> &SubPatternGroups {
            &self.profile
        }

        fn sub_pattern_at_index(&self, index: usize) -> &dyn SubPattern {
            &self.sub_patterns[index]
        }

        fn partitioning(&self) -> &Option<Partitioning> {
            unimplemented!()
        }
    }

    struct MockSubPattern(syn::Ident);

    impl SubPattern for MockSubPattern {
        fn get(&self) -> SubPatternDefinition {
            unimplemented!()
        }

        fn get_identifier(&self) -> &Ident {
            &self.0
        }
    }

    struct MockSubPatternCodeGen;

    impl SubPatternCodeGen for MockSubPatternCodeGen {
        fn generate(sub_pattern: &dyn SubPattern, _anonymous: bool) -> TokenStream {
            sub_pattern.get_identifier().to_token_stream()
        }
    }

    // Tests
    #[test]
    fn test_single() {
        let profile = SubPatternGroups(vec![Group::new(0)]);

        let bundle = MockCaseBundle {
            sub_patterns: vec![MockSubPattern(ident("A"))],
            profile,
        };

        let result =
            MatchArmCompiler::generate::<MockSubPatternCodeGen>(Span::call_site(), &bundle);

        assert_tokens!( result, {
            A => (0usize, 1usize),
        })
    }

    #[test]
    fn test_multiple_complex() {
        // Create mock sub-patterns
        let sub_patterns = vec![
            MockSubPattern(ident("A")),
            MockSubPattern(ident("B")),
            MockSubPattern(ident("A")),
            MockSubPattern(ident("C")),
            MockSubPattern(ident("B")),
            MockSubPattern(ident("D")),
            MockSubPattern(ident("A")),
            MockSubPattern(ident("C")),
            MockSubPattern(ident("E")),
            MockSubPattern(ident("B")),
            MockSubPattern(ident("D")),
            MockSubPattern(ident("A")),
        ];

        // Simulate a complex pattern: ["A", "B", "A", "C", "B", "D", "A", "C", "E", "B", "D", "A"]
        // Expected stats:
        // A: 4 occurrences, positions [0, 2, 6, 11]
        // B: 3 occurrences, positions [1, 4, 9]
        // C: 2 occurrences, positions [3, 7]
        // D: 2 occurrences, positions [5, 10]
        // E: 1 occurrence, position [8]
        let groups = vec![
            {
                let mut g = Group::new(0);
                g.push(2);
                g.push(6);
                g.push(11);
                g
            },
            {
                let mut g = Group::new(1);
                g.push(4);
                g.push(9);
                g
            },
            {
                let mut g = Group::new(3);
                g.push(7);
                g
            },
            {
                let mut g = Group::new(5);
                g.push(10);
                g
            },
            Group::new(8),
        ];
        let profile = SubPatternGroups(groups);

        let bundle = MockCaseBundle {
            sub_patterns,
            profile,
        };

        let result =
            MatchArmCompiler::generate::<MockSubPatternCodeGen>(Span::call_site(), &bundle);

        // The expected match arms, in order, with correct ranges
        assert_tokens!( result, {
            A => (0usize, 4usize),
            B => (4usize, 7usize),
            C => (7usize, 9usize),
            D => (9usize, 11usize),
            E => (11usize, 12usize),
        })
    }
}
