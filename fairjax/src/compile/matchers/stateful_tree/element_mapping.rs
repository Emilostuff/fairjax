use crate::compile::matchers::stateful_tree::profile::PatternProfile;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;

/// Generates element mappings that define subpattern positions for permutation algorithms
pub trait ElementMappingCodeGen {
    /// Generate code creating Element instances for each unique subpattern in the pattern
    fn generate(span: Span, profile: &PatternProfile) -> TokenStream;
}

pub struct ElementMappingCompiler;

impl ElementMappingCodeGen for ElementMappingCompiler {
    fn generate(span: Span, profile: &PatternProfile) -> TokenStream {
        // Generate Element instances containing position lists for each subpattern
        let element_mappings = profile.0.iter().map(|sp_stats| {
            let positions = sp_stats.positions.clone();

            // Generate a mapping for each occurence of the sub pattern
            (0..sp_stats.occurrences)
                .map(|_| {
                    quote_spanned! { span =>
                        fairjax_core::strategies::stateful_tree::permute::Element::new(
                            vec![#(#positions),*]
                        ),
                    }
                })
                .collect::<TokenStream>()
        });

        // Assemple element mappings
        quote_spanned!( span => [ #(#element_mappings)* ] )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::matchers::stateful_tree::profile::{PatternProfile, SubPatternStats};
    use crate::parse::sub_pattern::SubPattern;
    use proc_macro_utils::assert_tokens;

    // Mock objects for testing
    struct MockSubPattern;

    impl SubPattern for MockSubPattern {
        fn get(&self) -> crate::parse::sub_pattern::SubPatternDefinition {
            unimplemented!()
        }

        fn get_identifier(&self) -> &syn::Ident {
            unimplemented!()
        }
    }

    // Helper methods
    fn sub_pattern_stats(positions: Vec<usize>, occurrences: usize) -> SubPatternStats<'static> {
        SubPatternStats {
            sub_pattern: &MockSubPattern,
            positions,
            occurrences,
        }
    }

    #[test]
    fn test_generate_single_subpattern_single_occurrence() {
        // Create pattern profile
        let profile = PatternProfile(vec![sub_pattern_stats(vec![0], 1)]);

        // Generate result code
        let result = ElementMappingCompiler::generate(Span::call_site(), &profile);

        // Verfiy correctness
        #[rustfmt::skip]
        assert_tokens!(result, {
            [ fairjax_core::strategies::stateful_tree::permute::Element::new(vec![0usize]), ]
        });
    }

    #[test]
    fn test_generate_single_subpattern_multiple_occurrences() {
        // Create pattern profile
        let profile = PatternProfile(vec![sub_pattern_stats(vec![0, 1, 2, 3], 4)]);

        // Generate result code
        let result = ElementMappingCompiler::generate(Span::call_site(), &profile);

        // Verfiy correctness
        #[rustfmt::skip]
        assert_tokens!(result, {
            [
                fairjax_core::strategies::stateful_tree::permute::Element::new(vec![0usize, 1usize, 2usize, 3usize]),
                fairjax_core::strategies::stateful_tree::permute::Element::new(vec![0usize, 1usize, 2usize, 3usize]),
                fairjax_core::strategies::stateful_tree::permute::Element::new(vec![0usize, 1usize, 2usize, 3usize]),
                fairjax_core::strategies::stateful_tree::permute::Element::new(vec![0usize, 1usize, 2usize, 3usize]),
            ]
        });
    }

    #[test]
    fn test_generate_multiple_subpatterns_multiple_occurrences() {
        // Create pattern profile
        let profile = PatternProfile(vec![
            sub_pattern_stats(vec![0, 2], 2),
            sub_pattern_stats(vec![1, 3], 2),
        ]);

        // Generate result code
        let result = ElementMappingCompiler::generate(Span::call_site(), &profile);

        // Verfiy correctness
        #[rustfmt::skip]
        assert_tokens!(result, {
            [
                fairjax_core::strategies::stateful_tree::permute::Element::new(vec![0usize, 2usize]),
                fairjax_core::strategies::stateful_tree::permute::Element::new(vec![0usize, 2usize]),
                fairjax_core::strategies::stateful_tree::permute::Element::new(vec![1usize, 3usize]),
                fairjax_core::strategies::stateful_tree::permute::Element::new(vec![1usize, 3usize]),
            ]
        });
    }

    #[test]
    fn test_multiple_subpatterns_single_occurences() {
        // Create pattern profile
        let profile = PatternProfile(vec![
            sub_pattern_stats(vec![0], 1),
            sub_pattern_stats(vec![1], 1),
            sub_pattern_stats(vec![2], 1),
        ]);

        // Generate result code
        let result = ElementMappingCompiler::generate(Span::call_site(), &profile);

        // Verfiy correctness
        #[rustfmt::skip]
        assert_tokens!(result, {
            [
                fairjax_core::strategies::stateful_tree::permute::Element::new(vec![0usize]),
                fairjax_core::strategies::stateful_tree::permute::Element::new(vec![1usize]),
                fairjax_core::strategies::stateful_tree::permute::Element::new(vec![2usize]),
            ]
        });
    }
}
