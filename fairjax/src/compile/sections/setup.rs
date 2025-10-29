use crate::{compile::matchers::SetupCodeGen, parse::definition::Definition};
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;

pub trait SetupSectionCodeGen {
    fn generate<D: SetupCodeGen>(def: &dyn Definition) -> TokenStream;
}

pub struct SetupSection;

impl SetupSectionCodeGen for SetupSection {
    fn generate<S: SetupCodeGen>(def: &dyn Definition) -> TokenStream {
        let mailbox = def.context().mailbox;

        let identifiers: Vec<_> = def
            .cases()
            .iter()
            .map(|c| format_ident!("case{}", c.index()))
            .collect();

        let setups = identifiers
            .iter()
            .zip(def.cases().iter())
            .map(|(ident, case)| S::generate(*case, def.context(), ident))
            .collect::<Vec<TokenStream>>();

        quote! {
            #(
                #setups
                #mailbox.add_case(Box::new(#identifiers));
            )*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::context::Context;
    use proc_macro_utils::assert_tokens;
    use proc_macro2::TokenStream;
    use quote::{ToTokens, format_ident};

    // Mock Case
    struct MockCase {
        id: usize,
    }
    impl crate::parse::case::Case for MockCase {
        fn index(&self) -> usize {
            self.id
        }
        fn strategy(&self) -> crate::parse::strategy::Strategy {
            unimplemented!()
        }
        fn pattern(&self) -> &dyn crate::parse::pattern::Pattern {
            unimplemented!()
        }
        fn guard(&self) -> Option<syn::Expr> {
            unimplemented!()
        }
        fn body(&self) -> syn::Expr {
            unimplemented!()
        }
    }

    // Mock Definition
    struct MockDefinition {
        cases: Vec<MockCase>,
        context: Context,
    }
    impl crate::parse::definition::Definition for MockDefinition {
        fn cases(&self) -> Vec<&dyn crate::parse::case::Case> {
            self.cases
                .iter()
                .map(|c| c as &dyn crate::parse::case::Case)
                .collect()
        }
        fn context(&self) -> Context {
            self.context.clone()
        }
    }

    // Mock SetupCodeGen
    struct MockSetupCodeGen;
    impl SetupCodeGen for MockSetupCodeGen {
        fn generate(
            case: &dyn crate::parse::case::Case,
            _context: Context,
            ident: &syn::Ident,
        ) -> TokenStream {
            format_ident!("setup{}_{}", case.index(), ident).to_token_stream()
        }
    }

    #[test]
    fn test_single_case() {
        let def = MockDefinition {
            cases: vec![MockCase { id: 0 }],
            context: Context {
                incoming_message: syn::parse_quote! { msg },
                mailbox: syn::parse_quote! { mailbox },
                message_type: syn::parse_quote! { MyMessage },
            },
        };
        let result = SetupSection::generate::<MockSetupCodeGen>(&def);

        assert_tokens!(result, { setup0_case0 mailbox.add_case(Box::new(case0)); });
    }

    #[test]
    fn test_multiple_cases() {
        let def = MockDefinition {
            cases: vec![MockCase { id: 0 }, MockCase { id: 1 }, MockCase { id: 2 }],
            context: Context {
                incoming_message: syn::parse_quote! { msg },
                mailbox: syn::parse_quote! { mailbox },
                message_type: syn::parse_quote! { MyMessage },
            },
        };
        let result = SetupSection::generate::<MockSetupCodeGen>(&def);

        assert_tokens!(result, {
            setup0_case0 mailbox.add_case(Box::new(case0));
            setup1_case1 mailbox.add_case(Box::new(case1));
            setup2_case2 mailbox.add_case(Box::new(case2));
        });
    }
}
