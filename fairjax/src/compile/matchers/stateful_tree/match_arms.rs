use crate::compile::matchers::stateful_tree::profile::PatternProfile;
use crate::compile::pattern::sub::SubPatternCodeGen;
use proc_macro2::TokenStream;
use quote::quote;

pub trait MatchArmCodeGen {
    fn generate<S: SubPatternCodeGen>(profile: &PatternProfile) -> TokenStream;
}

pub struct MatchArmCompiler;

impl MatchArmCodeGen for MatchArmCompiler {
    fn generate<S: SubPatternCodeGen>(profile: &PatternProfile) -> TokenStream {
        let match_arms = profile.0.iter().scan(0, |position, stats| {
            // Create anonymized sub pattern
            let anonymized_sub_pattern = S::generate(stats.sub_pattern, true);

            // Determine which positions in the data structure should be reserved
            // for this sub-pattern type
            let start = *position;
            let end = start + stats.occurrences;

            // Update folding iterator
            *position = end;

            // Assemble code snippets into match arm
            Some(quote!(#anonymized_sub_pattern => (#start, #end)))
        });

        // Combine, and comma separate match arms
        quote!(#(#match_arms),*,)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::matchers::stateful_tree::profile::{PatternProfile, SubPatternStats};
    use crate::compile::pattern::sub::SubPatternCodeGen;
    use crate::parse::sub_pattern::{SubPattern, SubPatternDefinition};
    use proc_macro_utils::assert_tokens;
    use proc_macro2::TokenStream;
    use quote::ToTokens;
    use syn::Ident;

    fn ident(input: &str) -> Ident {
        Ident::new(input, proc_macro2::Span::call_site())
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
        let sp = MockSubPattern(ident("A"));
        let profile = PatternProfile(vec![SubPatternStats {
            sub_pattern: &sp,
            occurrences: 1,
            positions: vec![0],
        }]);

        let result = MatchArmCompiler::generate::<MockSubPatternCodeGen>(&profile);

        assert_tokens!( result, {
            A => (0usize, 1usize),
        })
    }

    #[test]
    fn test_multiple_complex() {
        // Create mock sub-patterns
        let sp_a = MockSubPattern(ident("A"));
        let sp_b = MockSubPattern(ident("B"));
        let sp_c = MockSubPattern(ident("C"));
        let sp_d = MockSubPattern(ident("D"));
        let sp_e = MockSubPattern(ident("E"));

        // Simulate a complex pattern: ["A", "B", "A", "C", "B", "D", "A", "C", "E", "B", "D", "A"]
        // Expected stats:
        // A: 4 occurrences, positions [0, 2, 6, 11]
        // B: 3 occurrences, positions [1, 4, 9]
        // C: 2 occurrences, positions [3, 7]
        // D: 2 occurrences, positions [5, 10]
        // E: 1 occurrence, position [8]
        let stats = vec![
            SubPatternStats {
                sub_pattern: &sp_a,
                occurrences: 4,
                positions: vec![0, 2, 6, 11],
            },
            SubPatternStats {
                sub_pattern: &sp_b,
                occurrences: 3,
                positions: vec![1, 4, 9],
            },
            SubPatternStats {
                sub_pattern: &sp_c,
                occurrences: 2,
                positions: vec![3, 7],
            },
            SubPatternStats {
                sub_pattern: &sp_d,
                occurrences: 2,
                positions: vec![5, 10],
            },
            SubPatternStats {
                sub_pattern: &sp_e,
                occurrences: 1,
                positions: vec![8],
            },
        ];
        let profile = PatternProfile(stats);

        let result = MatchArmCompiler::generate::<MockSubPatternCodeGen>(&profile);

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
