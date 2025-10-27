use crate::compile::matcher::brute_force::BruteForceMatcherGenerator;
use crate::compile::matcher::stateful_tree::StatefulTreeMatcherGenerator;
use crate::parse::case::Case;
use crate::parse::strategy::Strategy;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

pub struct CaseGenerator {
    case: Case,
    case_index: usize,
    message_type: Ident,
    mailbox_ident: Ident,
}

impl CaseGenerator {
    pub fn new(case: Case, case_index: usize, message_type: Ident, mailbox_ident: Ident) -> Self {
        Self {
            case,
            case_index,
            message_type,
            mailbox_ident,
        }
    }

    fn input_unpacking_code(
        &self,
        input_var: TokenStream,
        mapping_var: TokenStream,
    ) -> TokenStream {
        let indices = 0..self.case.pattern.len();
        quote! {
            #(#input_var[#mapping_var.get(#indices)]),*
        }
    }

    fn guard_fn_code(&self, guard_ident: Ident) -> TokenStream {
        let unpacking = self.input_unpacking_code(quote!(messages), quote!(mapping));
        let pattern = self.generate_full_pattern();
        let guard = self.case.guard.clone();
        let message_type = self.message_type.clone();
        let span = guard.span();
        let pattern_len = self.case.pattern.len();

        quote_spanned! {span=>
            fn #guard_ident(messages: &[&#message_type; #pattern_len], mapping: &fairjax_core::Mapping<#pattern_len>) -> Option<bool> {
                match (#unpacking) {
                    (#pattern) => Some(#guard),
                    _ => None,
                }
            }
        }
    }

    fn pattern_match_code(&self) -> TokenStream {
        self.generate_full_pattern()
    }

    pub fn generate_full_pattern(&self) -> TokenStream {
        let full_sub_patterns = self.case.pattern.0.iter().map(|e| e.full.clone());

        quote! {
            #(#full_sub_patterns),*
        }
    }
}

impl CaseGenerator {
    pub fn generate_declaration_code(&self) -> TokenStream {
        let struct_ident = format_ident!("FairjaxGenerated{}", self.case_index);
        let guard_ident = format_ident!("fairjax_pattern_guard_{}", self.case_index);

        let guard_code = self.guard_fn_code(guard_ident.clone());

        let (declaration_code, init_code) = match &self.case.strategy {
            Strategy::Auto | Strategy::StatefulTree => {
                let pattern_gen = StatefulTreeMatcherGenerator::new(
                    self.case.pattern.clone(),
                    self.message_type.clone(),
                    struct_ident.clone(),
                    guard_ident.clone(),
                );
                (
                    pattern_gen.generate_declaration_code(),
                    pattern_gen.generate_init_code(),
                )
            }
            Strategy::BruteForce => {
                let pattern_gen = BruteForceMatcherGenerator::new(
                    self.message_type.clone(),
                    guard_ident.clone(),
                    self.case.pattern.len(),
                );
                (
                    pattern_gen.generate_declaration_code(),
                    pattern_gen.generate_init_code(),
                )
            }
        };

        let mailbox_ident = self.mailbox_ident.clone();

        quote! {
            // Generate match group impl
            #declaration_code

            // Declare guard
            #guard_code

            // Generate pattern matcher
            let pm = #init_code;

            // Add to mailbox
            #mailbox_ident.add_case(Box::new(pm));
        }
    }

    pub fn generate_action_code(&self) -> TokenStream {
        let pattern = self.pattern_match_code();
        let body = &self.case.body;
        let tuple_ident = format_ident!("into_{}_tuple", self.case.pattern.len());

        quote! {
            match result.#tuple_ident() {
                (#pattern) => {
                    #body
                },
                _ => panic!("not good")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::compare_token_streams;
    use proc_macro2::{Ident, Span};
    use quote::quote;

    #[test]
    fn test_input_unpacking_code() {
        let generator = CaseGenerator::new(
            Case::new(quote!(A && B && C), quote!(), quote!()).unwrap(),
            0,
            Ident::new("x", Span::call_site()),
            Ident::new("y", Span::call_site()),
        );

        let output = generator.input_unpacking_code(quote!(input), quote!(mapping));
        let expected = quote!(
            input[mapping.get(0usize)],
            input[mapping.get(1usize)],
            input[mapping.get(2usize)]
        );

        compare_token_streams(&expected, &output);
    }

    #[test]
    fn test_generate_pattern_match_code() {
        let generator = CaseGenerator::new(
            Case::new(quote!(A(a, b) && B(_, c) && C(d)), quote!(), quote!()).unwrap(),
            0,
            Ident::new("x", Span::call_site()),
            Ident::new("y", Span::call_site()),
        );

        let output = generator.pattern_match_code();
        let expected = quote!(A(a, b), B(_, c), C(d));

        compare_token_streams(&expected, &output);
    }

    #[test]
    fn test_generate_action_code() {
        let generator = CaseGenerator::new(
            Case::new(
                quote!(A(a, b) && B(_, c) && C(d)),
                quote!(),
                quote!(println!("Success");),
            )
            .unwrap(),
            0,
            Ident::new("x", Span::call_site()),
            Ident::new("y", Span::call_site()),
        );

        let output = generator.generate_action_code();

        let expected = quote! {
            match result.into_3_tuple() {
                (A(a, b), B(_, c), C(d)) => {
                    println!("Success");
                },
                _ => panic!("not good")
            }
        };

        compare_token_streams(&expected, &output);
    }
}
