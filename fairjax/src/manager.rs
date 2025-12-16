use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Error, Expr, Ident, Lit, Result};

pub struct FairjaxManagerDefinition {
    pub matcher_count: usize,
}

// Input Parsing
impl FairjaxManagerDefinition {
    pub fn parse(input: TokenStream) -> Result<Self> {
        // Check that the matcher count is correctly defined
        let matcher_count_lit = match syn::parse2(input.clone()) {
            Ok(Expr::Lit(expr_lit)) => expr_lit.lit.clone(),
            _ => return Err(Error::new_spanned(input, "Expected `<MATCHER_COUNT>` here")),
        };

        // Validate that thematcher count is a usize > 0
        let error = Error::new_spanned(
            matcher_count_lit.clone(),
            "Expected `MATCHER_COUNT` to be a valid, nonzero usize integer",
        );

        let matcher_count = match matcher_count_lit {
            Lit::Int(lit_int) => match lit_int.base10_parse::<usize>() {
                Ok(count) if count > 0 => count,
                _ => return Err(error),
            },
            _ => return Err(error),
        };

        Ok(FairjaxManagerDefinition { matcher_count })
    }
}

// Top-level Codegen
impl FairjaxManagerDefinition {
    pub fn generate(self) -> TokenStream {
        // Define standardized struct and enum names
        let struct_ident = Ident::new("FairjaxManager", Span::call_site());
        let enum_ident = Ident::new("Matcher", Span::call_site());
        let enum_variant_names: Vec<_> = (0..self.matcher_count)
            .into_iter()
            .map(|i| format_ident!("Fairjax{}", i))
            .collect();

        // Generate code snippets
        let enum_code = Self::gen_enum_declaration_code(&enum_ident, &enum_variant_names);
        let struct_code =
            Self::gen_struct_declaration_code(&enum_ident, &struct_ident, self.matcher_count);
        let new_code = Self::gen_new_code(&enum_ident, &struct_ident, &enum_variant_names[0]);
        let active_matcher_code = Self::gen_active_matcher_code(&enum_ident, &enum_variant_names);
        let switch_to_code = Self::gen_switch_to_code(&enum_ident);

        // Combine everything
        quote! {
            // Declare active matcher enum and manager struct
            #enum_code

            #struct_code

            impl<T> #struct_ident<T> {
                #new_code

                #active_matcher_code

                #switch_to_code

                pub fn process_incoming(&self, message: T) {
                    self.queue.borrow_mut().push_back(message);
                }

                pub fn next(&self) -> Option<T> {
                    self.queue.borrow_mut().pop_front()
                }
            }
        }
    }
}

// Codegen Helpers
impl FairjaxManagerDefinition {
    fn gen_enum_declaration_code(
        enum_ident: &Ident,
        enum_variant_names: &Vec<Ident>,
    ) -> TokenStream {
        quote! {
            enum #enum_ident {
                #( #enum_variant_names ),* ,
            }
        }
    }

    fn gen_struct_declaration_code(
        enum_ident: &Ident,
        struct_ident: &Ident,
        matcher_count: usize,
    ) -> TokenStream {
        quote! {
            struct #struct_ident<T> {
                active_matcher: std::cell::RefCell<#enum_ident>,
                mailboxes: [std::cell::RefCell<fairjax_core::MailBox<T>>; #matcher_count],
                queue: std::cell::RefCell<std::collections::VecDeque<T>>,
            }

        }
    }

    fn gen_new_code(
        enum_ident: &Ident,
        struct_ident: &Ident,
        enum_variant_name: &Ident,
    ) -> TokenStream {
        quote! {
            pub fn new() -> Self {
                #struct_ident {
                    active_matcher: std::cell::RefCell::new(#enum_ident::#enum_variant_name),
                    mailboxes: std::array::from_fn(|_| std::cell::RefCell::new(fairjax_core::MailBox::<T>::default())),
                    queue: std::cell::RefCell::new(std::collections::VecDeque::new()),
                }
            }
        }
    }

    fn gen_switch_to_code(enum_ident: &Ident) -> TokenStream {
        quote! {
            pub fn switch_to(&self, target: #enum_ident, mut current_mailbox: std::cell::RefMut<fairjax_core::MailBox<T>>) {
                // Extract messages and push to processing queue
                let previous_messages = current_mailbox.extract();
                for m in previous_messages.into_iter().rev() {
                    self.queue.borrow_mut().push_front(m);
                }

                // Set active matcher:
                *self.active_matcher.borrow_mut() = target;
            }
        }
    }

    fn gen_active_matcher_code(enum_ident: &Ident, enum_variant_names: &Vec<Ident>) -> TokenStream {
        // Create idents and idx's for each matcher / enum variant
        let matcher_idxs = 0..enum_variant_names.len();

        quote! {
            pub fn active_matcher<'a>(&'a self) -> (#enum_ident, std::cell::RefMut<'a, fairjax_core::MailBox<T>>) {
                match *self.active_matcher.borrow() {
                    #( #enum_ident::#enum_variant_names => (#enum_ident::#enum_variant_names, self.mailboxes[#matcher_idxs].borrow_mut()) ),* ,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro_utils::assert_tokens;

    #[test]
    fn test_parse_valid_input() {
        // Define input
        let input = quote!(3);

        // Parse
        let result = FairjaxManagerDefinition::parse(input).unwrap();

        // Assert correctness
        assert_eq!(result.matcher_count, 3);
    }

    #[test]
    fn test_gen_enum_declaration_code() {
        // Define input
        let enum_ident = format_ident!("MyMatcher");
        let variant_idents = vec![
            format_ident!("Fairjax0"),
            format_ident!("Fairjax1"),
            format_ident!("Fairjax2"),
            format_ident!("Fairjax3"),
        ];

        // Perform codegen
        let result =
            FairjaxManagerDefinition::gen_enum_declaration_code(&enum_ident, &variant_idents);

        // Assert correctness
        assert_tokens!(result, {
            enum MyMatcher {
                Fairjax0,
                Fairjax1,
                Fairjax2,
                Fairjax3,
            }
        });
    }
}
