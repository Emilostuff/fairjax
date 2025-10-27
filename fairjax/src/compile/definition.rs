use crate::compile::case::CaseGenerator;
use crate::parse::definition::JoinDefinition;
use proc_macro2::TokenStream;
use quote::quote;

pub struct JoinDefinitionGenerator {
    def: JoinDefinition,
    case_generators: Vec<CaseGenerator>,
}

impl JoinDefinitionGenerator {
    pub fn new(def: JoinDefinition) -> Self {
        let case_generators = def
            .cases
            .iter()
            .enumerate()
            .map(|(i, c)| {
                CaseGenerator::new(c.clone(), i, def.message_type.clone(), def.mailbox.clone())
            })
            .collect();

        Self {
            def,
            case_generators,
        }
    }

    fn action_code(&self, match_result: TokenStream) -> TokenStream {
        let actions = self
            .case_generators
            .iter()
            .map(|g| g.generate_action_code())
            .collect::<Vec<TokenStream>>();
        let indices = 0..actions.len();

        quote! {
            match #match_result.case_id() {
                #(&fairjax_core::CaseId(#indices) => {#actions}),*,
                _ => panic!(),
            }
        }
    }

    fn declaration_code(&self) -> TokenStream {
        let declarations = self
            .case_generators
            .iter()
            .map(|g| g.generate_declaration_code())
            .collect::<Vec<TokenStream>>();

        quote! {
            #(#declarations)*
        }
    }
}

// Code gen endpoints
impl JoinDefinitionGenerator {
    pub fn generate(self) -> TokenStream {
        // Init mailbox if not done
        let mailbox_ident = self.def.mailbox.clone();
        let match_result = quote!(result);

        let action_code = self.action_code(match_result.clone());
        let declaration_code = self.declaration_code();
        let msg = self.def.message;

        quote! {
            // Run init once
            if !#mailbox_ident.is_initialized() {
                // Check if user modified mailbox
                if #mailbox_ident.is_modified() {
                    panic!("Mailbox was modified prior to initialization");
                }
                #declaration_code

                // finalize init
                #mailbox_ident.init();
            }

            // Run action if match was found
            if let Some(#match_result) = #mailbox_ident.process(#msg) {
                #action_code
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::case::Case;
    use crate::utils::compare_token_streams;
    use proc_macro2::{Ident, Span};
    use quote::quote;

    #[test]
    fn test_generate_action_code() {
        let case_0 = Case::new(
            quote!(A(a, b) && B(_, c) && C(d)),
            quote!(),
            quote! {
                println!("Success");
            },
        )
        .unwrap();

        let case_1 = Case::new(
            quote!(E(k, _) && C(d)),
            quote!(),
            quote! {
                println!("More Success");
            },
        )
        .unwrap();

        let generator = JoinDefinitionGenerator::new(JoinDefinition {
            message_type: Ident::new("dont_care_1", Span::call_site()),
            mailbox: Ident::new("dont_care_2", Span::call_site()),
            message: Ident::new("dont_care_3", Span::call_site()),
            cases: vec![case_0, case_1],
        });

        let match_result = quote!(result);

        let output = generator.action_code(match_result.clone());
        let expected = quote! {
            match result.case_id() {
                &fairjax_core::CaseId(0usize) => {
                    match result.into_3_tuple() {
                        (A(a, b), B(_, c), C(d)) => {
                            println!("Success");
                        },
                        _ => panic!("not good")
                    }
                },
                &fairjax_core::CaseId(1usize) => {
                    match result.into_2_tuple() {
                        (E(k, _), C(d)) => {
                            println!("More Success");
                        },
                        _ => panic!("not good")
                    }
                },
                _ => panic!(),
            }
        };

        compare_token_streams(&expected, &output);
    }
}
