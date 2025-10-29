use crate::compile::case::action::Action;
use crate::compile::matchers::Setup;
use crate::compile::sections::action::ActionSectionCodeGen;
use crate::compile::sections::setup::SetupSectionCodeGen;
use crate::parse::{context::Context, definition::Definition};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub trait TopLevelCodeGen {
    fn generate<A: ActionSectionCodeGen, D: SetupSectionCodeGen>(
        def: &dyn Definition,
    ) -> TokenStream;
}

pub struct TopLevel;

impl TopLevel {}

impl TopLevelCodeGen for TopLevel {
    fn generate<A: ActionSectionCodeGen, D: SetupSectionCodeGen>(
        def: &dyn Definition,
    ) -> TokenStream {
        let Context {
            incoming_message,
            mailbox,
            ..
        } = def.context();

        let match_result = Ident::new("result", Span::call_site());
        let action_section = A::generate::<Action>(def.cases(), &match_result);
        let setup_section = D::generate::<Setup>(def);

        quote! {
            // Run init once
            if !#mailbox.is_initialized() {
                // Check if user modified mailbox
                if #mailbox.is_modified() {
                    panic!("Mailbox was modified prior to initialization");
                }

                // Declare all matchers
                #setup_section

                // Finalize init
                #mailbox.init();
            }

            // Run action if match was found
            if let Some(#match_result) = #mailbox.process(#incoming_message) {
                #action_section
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use proc_macro_utils::assert_tokens;

//     struct DefDI;

//     impl Definition for DefDI {
//         fn context(&self) -> Context {
//             unimplemented!()
//         }
//         fn cases(&self) -> Vec<Box<dyn Case>> {
//             Vec::new()
//         }
//     }

//     struct Act;

//     impl ActionSectionCodeGen for Act {
//         fn generate(cases: Vec<Box<dyn Case>>) -> TokenStream {
//             TokenStream::from(quote!(actionblock))
//         }
//     }

//     #[test]
//     fn test_top_level_generate() {
//         let result = TopLevel::generate::<Act>(DefDI);

//         assert_tokens!(result, { actionblock });
//     }
// }
