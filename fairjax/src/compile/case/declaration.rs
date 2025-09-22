use crate::Compile;
use crate::parse::case::Case;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

pub struct CaseDeclarationGenerator {
    case: Case,
    case_index: usize,
    message_type: Ident,
    mailbox_ident: Ident,
}

impl CaseDeclarationGenerator {
    pub fn new(case: Case, case_index: usize, message_type: Ident, mailbox_ident: Ident) -> Self {
        Self {
            case,
            case_index,
            message_type,
            mailbox_ident,
        }
    }

    fn input_unpacking_code(&self, input_var: TokenStream) -> TokenStream {
        let indices = 0..self.case.pattern.len();
        quote! {
            #(#input_var[#indices]),*
        }
    }

    fn guard_fn_code(&self, guard_ident: Ident) -> TokenStream {
        let unpacking = self.input_unpacking_code(quote!(messages));
        let pattern = self.case.pattern.generate_full_pattern();
        let guard = self.case.guard.clone();
        let message_type = self.message_type.clone();
        let span = guard.span();

        quote_spanned! {span=>
            fn #guard_ident(messages: &Vec<&#message_type>) -> bool {
                match (#unpacking) {
                    (#pattern) => #guard,
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl Compile for CaseDeclarationGenerator {
    fn generate(self) -> TokenStream {
        let match_group_ident = format_ident!("FairjaxGenerated{}", self.case_index);
        let guard_ident = format_ident!("fairjax_pattern_guard_{}", self.case_index);

        let guard_code = self.guard_fn_code(guard_ident.clone());
        let declaration_code = self
            .case
            .pattern
            .generate_declaration_code(self.message_type.clone(), match_group_ident.clone());

        let message_type = self.message_type;
        let mailbox_ident = self.mailbox_ident;

        quote! {
            // Generate match group impl
            #declaration_code

            // Declare guard
            #guard_code

            // Generate pattern matcher
            let pm = fairjax_core::pattern::PatternMatcher::<#match_group_ident, #message_type>::new(#guard_ident);

            // Add to mailbox
            #mailbox_ident.add_pattern(Box::new(pm));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::pattern::Pattern;
    use crate::utils::compare_token_streams;
    use proc_macro2::{Ident, Span};
    use quote::quote;

    #[test]
    fn test_input_unpacking_code() {
        let generator = CaseDeclarationGenerator::new(
            Case {
                pattern: Pattern::parse(quote!(A && B && C)).unwrap(),
                guard: quote!(),
                body: quote!(),
            },
            0,
            Ident::new("x", Span::call_site()),
            Ident::new("y", Span::call_site()),
        );

        let output = generator.input_unpacking_code(quote!(input));
        let expected = quote!(input[0usize], input[1usize], input[2usize]);

        compare_token_streams(&expected, &output);
    }
}
