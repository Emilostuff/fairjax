use crate::{
    compile::pattern::{full::PatternCodeGen, sub::SubPatternCompiler},
    parse::case::Case,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

pub trait ActionCodeGen {
    fn generate<P: PatternCodeGen>(case: &dyn Case, result_ident: &Ident) -> TokenStream;
}

pub struct Action;

impl ActionCodeGen for Action {
    fn generate<P: PatternCodeGen>(case: &dyn Case, result_ident: &Ident) -> TokenStream {
        let tuple_ident = format_ident!("into_{}", case.pattern().len());
        let pattern_match_code = P::generate::<SubPatternCompiler>(case.pattern());
        let body = case.body();

        quote! {
            match #result_ident.#tuple_ident() {
                #pattern_match_code => #body,
                _ => panic!("not good"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::pattern::sub::SubPatternCodeGen;
    use crate::parse::case::Case;
    use crate::parse::pattern::Pattern;
    use crate::parse::strategy::Strategy;
    use crate::parse::sub_pattern::SubPattern;
    use proc_macro_utils::assert_tokens;
    use proc_macro2::TokenStream;
    use quote::ToTokens;
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
    }

    // Mock Case with N sub-patterns
    struct MockCase {
        pattern: MockPattern,
    }

    impl Case for MockCase {
        fn index(&self) -> usize {
            unimplemented!()
        }
        fn strategy(&self) -> Strategy {
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

        let generated = Action::generate::<MockPatternCodeGen>(&case, &format_ident!("result"));

        assert_tokens!(generated, {
            match result.into_1() {
                SIZE_1 => BODY,
                _ => panic!("not good"),
            }
        });
    }

    #[test]
    fn test_generate_pattern_size_2() {
        let case = MockCase {
            pattern: MockPattern(2),
        };

        let generated = Action::generate::<MockPatternCodeGen>(&case, &format_ident!("result"));

        assert_tokens!(generated, {
            match result.into_2() {
                SIZE_2 => BODY,
                _ => panic!("not good"),
            }
        });
    }

    #[test]
    fn test_generate_pattern_size_5() {
        let case = MockCase {
            pattern: MockPattern(5),
        };

        let generated = Action::generate::<MockPatternCodeGen>(&case, &format_ident!("result"));

        assert_tokens!(generated, {
            match result.into_5() {
                SIZE_5 => BODY,
                _ => panic!("not good"),
            }
        });
    }
}
