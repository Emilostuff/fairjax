use crate::compile::pattern::sub::SubPatternCodeGen;
use crate::parse::pattern::Pattern;
use proc_macro2::TokenStream;
use quote::quote_spanned;

pub trait PatternCodeGen {
    fn generate<S: SubPatternCodeGen>(pattern: &dyn Pattern) -> TokenStream;
}

pub struct PatternCompiler;

impl PatternCodeGen for PatternCompiler {
    fn generate<S: SubPatternCodeGen>(pattern: &dyn Pattern) -> TokenStream {
        // Perform code gen on all SubPatterns
        let sub_patterns = pattern
            .sub_patterns()
            .into_iter()
            .map(|sp| S::generate(&*sp, false))
            .collect::<Vec<TokenStream>>();

        // If there is only on SubPattern, omit parenthesis around it
        if sub_patterns.len() == 1 {
            let sub_pattern = sub_patterns[0].clone();
            return quote_spanned!( pattern.span() => #sub_pattern );
        }

        // Otherwise return a list of comma separated SubPatterns, enclosed in parenthesis
        quote_spanned!( pattern.span() => (#(#sub_patterns),*) )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::pattern::sub::SubPatternCodeGen;
    use crate::parse::sub_pattern::{SubPattern, SubPatternDefinition};
    use proc_macro_utils::assert_tokens;
    use proc_macro2::{Span, TokenStream};
    use quote::quote;
    use syn::Ident;

    // Define Dependency Injection objects for testing
    struct MockSubPattern;

    impl SubPattern for MockSubPattern {
        fn get(&self) -> SubPatternDefinition {
            unimplemented!()
        }

        fn get_identifier(&self) -> &Ident {
            unimplemented!()
        }
    }

    struct MockPattern {
        sub_patterns: Vec<MockSubPattern>,
    }

    impl MockPattern {
        fn new(pattern_size: usize) -> Self {
            Self {
                sub_patterns: (0..pattern_size).map(|_| MockSubPattern).collect(),
            }
        }
    }

    impl Pattern for MockPattern {
        fn sub_patterns(&self) -> Vec<&dyn SubPattern> {
            self.sub_patterns
                .iter()
                .map(|sp| sp as &dyn SubPattern)
                .collect()
        }

        fn len(&self) -> usize {
            unimplemented!()
        }

        fn span(&self) -> Span {
            Span::call_site()
        }
    }

    struct MockSubPatternCodeGen;

    impl SubPatternCodeGen for MockSubPatternCodeGen {
        fn generate(_mock_pattern: &dyn SubPattern, _anonymous: bool) -> TokenStream {
            quote!(mock_sp_ident)
        }
    }

    // Tests
    #[test]
    fn test_single_sub_pattern() {
        let pattern = MockPattern::new(1);
        let result = PatternCompiler::generate::<MockSubPatternCodeGen>(&pattern);

        assert_tokens!(result, { mock_sp_ident });
    }

    #[test]
    fn test_multiple_sub_patterns() {
        let pattern = MockPattern::new(3);
        let result = PatternCompiler::generate::<MockSubPatternCodeGen>(&pattern);

        assert_tokens!(result, { (mock_sp_ident, mock_sp_ident, mock_sp_ident) });
    }
}
