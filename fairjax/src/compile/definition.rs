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
        let input_var = quote!(input);

        let case_generators = def
            .cases
            .iter()
            .enumerate()
            .map(|(i, c)| {
                CaseGenerator::new(
                    c.clone(),
                    i,
                    def.message_type.clone(),
                    def.mailbox.clone(),
                    input_var.clone(),
                )
            })
            .collect();

        Self {
            def,
            case_generators,
        }
    }

    fn action_code(&self, match_result: TokenStream) -> TokenStream {
        let input_var = quote!(input);
        let actions = self
            .case_generators
            .iter()
            .map(|g| g.generate_action_code())
            .collect::<Vec<TokenStream>>();
        let indices = 0..actions.len();

        quote! {
            match #match_result {
                #(Some((#indices, #input_var)) => {#actions}),*,
                None => (),
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

            let #match_result = #mailbox_ident.consume(#msg.clone());

            // Run action if match was found
            #action_code
        }
    }
}

#[cfg(test)]
mod join_definition_generator_tests {
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
            message_type: Ident::new("x", Span::call_site()),
            mailbox: Ident::new("y", Span::call_site()),
            message: Ident::new("z", Span::call_site()),
            cases: vec![case_0, case_1],
        });

        let match_result = quote!(result);

        let output = generator.action_code(match_result.clone());
        let expected = quote! {
            match result {
                Some((0usize, input)) => {
                    match (input[0usize], input[1usize], input[2usize]) {
                        (A(a, b), B(_, c), C(d)) => {
                            println!("Success");
                        },
                        _ => panic!("not good")
                    }
                },
                Some((1usize, input)) => {
                    match (input[0usize], input[1usize]) {
                        (E(k, _), C(d)) => {
                            println!("More Success");
                        },
                        _ => panic!("not good")
                    }
                },
                None => (),
                _ => panic!(),
            }
        };

        compare_token_streams(&expected, &output);
    }
}
