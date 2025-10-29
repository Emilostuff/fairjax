use crate::{
    compile::{case::action::ActionCodeGen, pattern::full::PatternCompiler},
    parse::case::Case,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub trait ActionSectionCodeGen {
    fn generate<A: ActionCodeGen>(cases: Vec<&dyn Case>, result_ident: &Ident) -> TokenStream;
}

pub struct ActionSection;

impl ActionSectionCodeGen for ActionSection {
    fn generate<A: ActionCodeGen>(cases: Vec<&dyn Case>, result_ident: &Ident) -> TokenStream {
        let actions = cases
            .iter()
            .map(|c| A::generate::<PatternCompiler>(*c, result_ident))
            .collect::<Vec<TokenStream>>();

        let indices = cases.iter().map(|c| c.index());

        quote! {
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
    use proc_macro_utils::assert_tokens;
    use proc_macro2::TokenStream;
    use quote::{ToTokens, format_ident};
    use syn::Expr;

    // Mock Case
    struct MockCase {
        id: usize,
    }

    impl Case for MockCase {
        fn index(&self) -> usize {
            self.id
        }
        fn strategy(&self) -> crate::parse::strategy::Strategy {
            unimplemented!()
        }
        fn pattern(&self) -> &dyn crate::parse::pattern::Pattern {
            unimplemented!()
        }
        fn guard(&self) -> Option<Expr> {
            unimplemented!()
        }
        fn body(&self) -> Expr {
            unimplemented!()
        }
    }

    // Mock ActionCodeGen
    struct MockActionCodeGen;

    impl ActionCodeGen for MockActionCodeGen {
        fn generate<P: crate::compile::pattern::full::PatternCodeGen>(
            case: &dyn Case,
            _: &syn::Ident,
        ) -> TokenStream {
            format_ident!("ACTION_{}", case.index()).to_token_stream()
        }
    }

    #[test]
    fn test_generate_single_case() {
        let case = MockCase { id: 0 };

        let generated =
            ActionSection::generate::<MockActionCodeGen>(vec![&case], &format_ident!("result"));

        assert_tokens!(generated, {
            match result.case_id() {
                &fairjax_core::CaseId(0usize) => ACTION_0,
                _ => panic!(),
            }
        })
    }

    #[test]
    fn test_generate_3_cases() {
        let cases = vec![MockCase { id: 0 }, MockCase { id: 1 }, MockCase { id: 2 }];

        let generated = ActionSection::generate::<MockActionCodeGen>(
            cases.iter().map(|c| c as &dyn Case).collect(),
            &format_ident!("result"),
        );

        assert_tokens!(generated, {
            match result.case_id() {
                &fairjax_core::CaseId(0usize) => ACTION_0,
                &fairjax_core::CaseId(1usize) => ACTION_1,
                &fairjax_core::CaseId(2usize) => ACTION_2,
                _ => panic!(),
            }
        })
    }
}
