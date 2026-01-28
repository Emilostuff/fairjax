use crate::compile::pattern::{full::PatternCodeGen, sub::SubPatternCompiler};
use crate::traits::Case;
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::Ident;

pub trait ActionCodeGen {
    fn generate<P: PatternCodeGen>(case: &dyn Case, result_name: &'static str) -> TokenStream;
}

pub struct Action;

impl ActionCodeGen for Action {
    fn generate<P: PatternCodeGen>(case: &dyn Case, result_name: &'static str) -> TokenStream {
        let tuple_ident = Ident::new(&format!("into_{}", case.pattern().len()), case.span());
        let result_ident = Ident::new(result_name, case.span());

        let pattern_match_code = P::generate::<SubPatternCompiler>(case.pattern());
        let body = case.body();

        quote_spanned! { case.span() =>
            match #result_ident.#tuple_ident() {
                #pattern_match_code => #body,
                _ => panic!("A critical error has occured!"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::pattern::sub::SubPatternCodeGen;
    use crate::traits::{Case, Pattern, SubPattern};
    use proc_macro_utils::assert_tokens;
    use proc_macro2::{Span, TokenStream};
    use quote::{ToTokens, format_ident};
    use syn::Expr;

    // Mock Pattern with n sub-patterns
    struct MockPattern(usize);

    impl Pattern for MockPattern {
        fn sub_patterns(&self) -> Vec<&dyn SubPattern> {
            unimplemented!()
        }

        fn len(&self) -> usize {
            self.0
        }

        fn span(&self) -> Span {
            Span::call_site()
        }
    }

    // Mock Case with N sub-patterns
    struct MockCase {
        pattern: MockPattern,
    }

    impl Case for MockCase {
        fn index(&self) -> usize {
            unimplemented!()
        }
        fn pattern(&self) -> &dyn Pattern {
            &self.pattern
        }
        fn guard(&self) -> Option<Expr> {
            unimplemented!()
        }
        fn body(&self) -> Expr {
            syn::parse_quote!(BODY)
        }

        fn span(&self) -> Span {
            Span::call_site()
        }
        fn ident_with_case_id(&self, _name: &'static str) -> Ident {
            unimplemented!()
        }
    }

    // Mock PatternCodeGen trait
    struct MockPatternCodeGen;

    impl PatternCodeGen for MockPatternCodeGen {
        fn generate<P: SubPatternCodeGen>(pattern: &dyn Pattern) -> TokenStream {
            // Just write the number of sub-patterns
            format_ident!("SIZE_{}", pattern.len()).to_token_stream()
        }
    }

    #[test]
    fn test_generate_pattern_size_1() {
        let case = MockCase {
            pattern: MockPattern(1),
        };

        let generated = Action::generate::<MockPatternCodeGen>(&case, "result");

        assert_tokens!(generated, {
            match result.into_1() {
                SIZE_1 => BODY,
                _ => panic!("A critical error has occured!"),
            }
        });
    }

    #[test]
    fn test_generate_pattern_size_2() {
        let case = MockCase {
            pattern: MockPattern(2),
        };

        let generated = Action::generate::<MockPatternCodeGen>(&case, "result");

        assert_tokens!(generated, {
            match result.into_2() {
                SIZE_2 => BODY,
                _ => panic!("A critical error has occured!"),
            }
        });
    }

    #[test]
    fn test_generate_pattern_size_5() {
        let case = MockCase {
            pattern: MockPattern(5),
        };

        let generated = Action::generate::<MockPatternCodeGen>(&case, "result");

        assert_tokens!(generated, {
            match result.into_5() {
                SIZE_5 => BODY,
                _ => panic!("A critical error has occured!"),
            }
        });
    }
}
