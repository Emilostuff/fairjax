use crate::compile::matchers::SetupCodeGen;
use crate::traits::Definition;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;

pub trait SetupSectionCodeGen {
    fn generate<D: SetupCodeGen>(def: &dyn Definition) -> TokenStream;
}

pub struct SetupSection;

impl SetupSectionCodeGen for SetupSection {
    fn generate<S: SetupCodeGen>(def: &dyn Definition) -> TokenStream {
        let mailbox = def.context().mailbox;

        let matcher_factories: Vec<_> = def
            .cases()
            .iter()
            .map(|cb| syn::Ident::new(&format!("matcher{}", cb.case().index()), cb.case().span()))
            .collect();

        let setups = matcher_factories
            .iter()
            .zip(def.cases().iter())
            .map(|(ident, case)| S::generate(*case, def.context(), ident))
            .collect::<Vec<TokenStream>>();

        quote_spanned! { Span::call_site() =>
            #(
                #setups
                #mailbox.add_case(#matcher_factories());
            )*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyse::profile::PatternProfile;
    use crate::analyse::strategy::Strategy;
    use crate::parse::context::Context;
    use crate::traits::{Case, CaseBundle, Definition, Pattern, SubPattern};
    use proc_macro_utils::assert_tokens;
    use proc_macro2::{Span, TokenStream};
    use quote::{ToTokens, format_ident};

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
        fn guard(&self) -> Option<syn::Expr> {
            unimplemented!()
        }
        fn body(&self) -> syn::Expr {
            unimplemented!()
        }
        fn span(&self) -> Span {
            Span::call_site()
        }
    }

    // Mock Definition
    struct MockDefinition {
        cases: Vec<MockCaseBundle>,
        context: Context,
    }
    impl Definition for MockDefinition {
        fn cases(&self) -> Vec<&dyn CaseBundle> {
            self.cases.iter().map(|c| c as &dyn CaseBundle).collect()
        }
        fn context(&self) -> Context {
            self.context.clone()
        }
    }

    // Mock SetupCodeGen
    struct MockSetupCodeGen;
    impl SetupCodeGen for MockSetupCodeGen {
        fn generate(case: &dyn CaseBundle, _context: Context, ident: &syn::Ident) -> TokenStream {
            format_ident!("setup{}_{}", case.case().index(), ident).to_token_stream()
        }
    }

    #[test]
    fn test_single_case() {
        let def = MockDefinition {
            cases: vec![MockCaseBundle {
                case: MockCase { id: 0 },
            }],
            context: Context {
                incoming_message: syn::parse_quote! { msg },
                mailbox: syn::parse_quote! { mailbox },
                message_type: syn::parse_quote! { MyMessage },
            },
        };
        let result = SetupSection::generate::<MockSetupCodeGen>(&def);

        assert_tokens!(result, { setup0_matcher0 mailbox.add_case(matcher0()); });
    }

    #[test]
    fn test_multiple_cases() {
        let def = MockDefinition {
            cases: vec![
                MockCaseBundle {
                    case: MockCase { id: 0 },
                },
                MockCaseBundle {
                    case: MockCase { id: 1 },
                },
                MockCaseBundle {
                    case: MockCase { id: 2 },
                },
            ],
            context: Context {
                incoming_message: syn::parse_quote! { msg },
                mailbox: syn::parse_quote! { mailbox },
                message_type: syn::parse_quote! { MyMessage },
            },
        };
        let result = SetupSection::generate::<MockSetupCodeGen>(&def);

        assert_tokens!(result, {
            setup0_matcher0 mailbox.add_case(matcher0());
            setup1_matcher1 mailbox.add_case(matcher1());
            setup2_matcher2 mailbox.add_case(matcher2());
        });
    }
}
