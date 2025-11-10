use crate::compile::{case::action::ActionCodeGen, pattern::full::PatternCompiler};
use crate::traits::CaseBundle;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub trait ActionSectionCodeGen {
    fn generate<A: ActionCodeGen>(
        cases: Vec<&dyn CaseBundle>,
        result_name: &'static str,
    ) -> TokenStream;
}

pub struct ActionSection;

impl ActionSectionCodeGen for ActionSection {
    fn generate<A: ActionCodeGen>(
        cases: Vec<&dyn CaseBundle>,
        result_name: &'static str,
    ) -> TokenStream {
        let actions = cases
            .iter()
            .map(|cb| A::generate::<PatternCompiler>(cb.case(), result_name))
            .collect::<Vec<TokenStream>>();

        let indices = cases.iter().map(|cb| cb.case().index());
        let result_ident = Ident::new(result_name, Span::call_site());

        quote! {
            #[allow(unused_variables)]
            match #result_ident.case_id() {
                #(&fairjax_core::CaseId(#indices) => #actions),*,
                _ => panic!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyse::profile::PatternProfile;
    use crate::analyse::strategy::Strategy;
    use crate::traits::{Case, Pattern, SubPattern};
    use proc_macro_utils::assert_tokens;
    use proc_macro2::TokenStream;
    use quote::{ToTokens, format_ident};
    use syn::Expr;

    struct MockCaseBundle {
        case: MockCase,
    }

    impl CaseBundle for MockCaseBundle {
        fn case(&self) -> &dyn Case {
            &self.case
        }
        fn strategy(&self) -> &Strategy {
            unimplemented!()
        }

        fn pattern_profile(&self) -> &PatternProfile {
            unimplemented!()
        }

        fn sub_pattern_at_index(&self, _index: usize) -> &dyn SubPattern {
            unimplemented!()
        }
    }

    // Mock Case
    struct MockCase {
        id: usize,
    }

    impl Case for MockCase {
        fn index(&self) -> usize {
            self.id
        }
        fn pattern(&self) -> &dyn Pattern {
            unimplemented!()
        }
        fn guard(&self) -> Option<Expr> {
            unimplemented!()
        }
        fn body(&self) -> Expr {
            unimplemented!()
        }

        fn span(&self) -> Span {
            Span::call_site()
        }
    }

    // Mock ActionCodeGen
    struct MockActionCodeGen;

    impl ActionCodeGen for MockActionCodeGen {
        fn generate<P: crate::compile::pattern::full::PatternCodeGen>(
            case: &dyn Case,
            _result_name: &'static str,
        ) -> TokenStream {
            format_ident!("ACTION_{}", case.index()).to_token_stream()
        }
    }

    #[test]
    fn test_generate_single_case() {
        let case = MockCase { id: 0 };
        let bundle = MockCaseBundle { case: case };

        let generated = ActionSection::generate::<MockActionCodeGen>(vec![&bundle], "result");

        assert_tokens!(generated, {
            #[allow(unused_variables)]
            match result.case_id() {
                &fairjax_core::CaseId(0usize) => ACTION_0,
                _ => panic!(),
            }
        })
    }

    #[test]
    fn test_generate_3_cases() {
        let cases = vec![MockCase { id: 0 }, MockCase { id: 1 }, MockCase { id: 2 }];
        let bundles: Vec<_> = cases
            .into_iter()
            .map(|c| MockCaseBundle { case: c })
            .collect();

        let generated = ActionSection::generate::<MockActionCodeGen>(
            bundles.iter().map(|c| c as &dyn CaseBundle).collect(),
            "result",
        );

        assert_tokens!(generated, {
            #[allow(unused_variables)]
            match result.case_id() {
                &fairjax_core::CaseId(0usize) => ACTION_0,
                &fairjax_core::CaseId(1usize) => ACTION_1,
                &fairjax_core::CaseId(2usize) => ACTION_2,
                _ => panic!(),
            }
        })
    }
}
