use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Expr, Ident, Lit, Path, Result, Token};

pub struct FairjaxManagerDefinition {
    pub manager_name: Path,
    pub message_type: Path,
    pub matcher_count: usize,
}

// Input Parsing
impl Parse for FairjaxManagerDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse input as comma separated list
        let args: Punctuated<Expr, Token![,]> = Punctuated::parse_terminated(input)?;

        // Check that the correct number of inputs were provided:
        if args.len() != 3 {
            return Err(Error::new(
                input.span(),
                "Expected input: `<MANAGER_NAME>, <MESSAGE_TYPE>, <MATCHER_COUNT>`",
            ));
        }

        // Check that the manager name is correctly defined
        let manager_name = match &args[0] {
            Expr::Path(expr_path) if expr_path.path.segments.len() == 1 => expr_path.path.clone(),
            other => return Err(Error::new_spanned(other, "Expected `<MANAGER_NAME>` here")),
        };

        // Check that the message type is correctly defined
        let message_type = match &args[1] {
            Expr::Path(expr_path) => expr_path.path.clone(),
            other => return Err(Error::new_spanned(other, "Expected `<MANAGER_NAME>` here")),
        };

        // Check that the mailbox count is correctly defined
        let matcher_count_lit = match &args[2] {
            Expr::Lit(expr_lit) => expr_lit.lit.clone(),
            other => return Err(Error::new_spanned(other, "Expected `<MATCHER_COUNT>` here")),
        };

        let error = Error::new_spanned(
            matcher_count_lit.clone(),
            "Expected `MATCHER_COUNT` to be a valid usize integer",
        );
        let matcher_count = match matcher_count_lit.clone() {
            Lit::Int(lit_int) => lit_int.base10_parse::<usize>().map_err(|_| error)?,
            _ => return Err(error),
        };

        Ok(FairjaxManagerDefinition {
            manager_name,
            message_type,
            matcher_count,
        })
    }
}

// Top-level Codegen
impl FairjaxManagerDefinition {
    pub fn generate(self) -> TokenStream {
        // Define standardized struct and enum names
        let enum_ident = Ident::new("ActiveMatcher", Span::call_site());
        let struct_ident = Ident::new("FairjaxManager", Span::call_site());

        // Generate code snippets
        let enum_code = Self::gen_enum_declaration_code(&enum_ident, self.matcher_count);
        let struct_code = Self::gen_struct_declaration_code(&enum_ident, &struct_ident);
        let new_code = Self::gen_new_code(&enum_ident, &struct_ident);
        let active_matcher_code = Self::gen_active_matcher_code(&enum_ident, self.matcher_count);
        let switch_to_code = Self::gen_switch_to_code(&enum_ident);

        // Retrieve values for code block
        let message_type = self.message_type;
        let manager_name = self.manager_name;
        let matcher_count = self.matcher_count;

        // Combine everything
        quote! {
            // Declare active matcher enum and manager struct
            #enum_code

            #struct_code

            impl<T, const N: usize> #struct_ident<T, N> {
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

            // Init manager
            let #manager_name = FairjaxManager::<#message_type, #matcher_count>::new();
        }
    }
}

// Codegen Helpers
impl FairjaxManagerDefinition {
    fn gen_enum_declaration_code(enum_ident: &Ident, matcher_count: usize) -> TokenStream {
        // Create idents for each matcher / enum variant
        let matchers: Vec<_> = (0..matcher_count)
            .into_iter()
            .map(|i| format_ident!("Matcher{}", i))
            .collect();

        quote! {
            enum #enum_ident {
                #( #matchers ),* ,
            }
        }
    }

    fn gen_struct_declaration_code(enum_ident: &Ident, struct_ident: &Ident) -> TokenStream {
        quote! {
            struct #struct_ident<T, const N: usize> {
                active_matcher: RefCell<#enum_ident>,
                mailboxes: [RefCell<fairjax_core::MailBox<T>>; N],
                queue: RefCell<VecDeque<T>>,
            }

        }
    }

    fn gen_new_code(enum_ident: &Ident, struct_ident: &Ident) -> TokenStream {
        quote! {
            pub fn new() -> Self {
                #struct_ident {
                    active_matcher: RefCell::new(#enum_ident::Matcher0),
                    mailboxes: array::from_fn(|_| RefCell::new(fairjax_core::MailBox::<T>::default())),
                    queue: RefCell::new(VecDeque::new()),
                }
            }
        }
    }

    fn gen_switch_to_code(enum_ident: &Ident) -> TokenStream {
        quote! {
            pub fn switch_to(&self, target: #enum_ident, mut current_mailbox: RefMut<fairjax_core::MailBox<T>>) {
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

    fn gen_active_matcher_code(enum_ident: &Ident, matcher_count: usize) -> TokenStream {
        // Create idents and idx's for each matcher / enum variant
        let matcher_idxs = 0..matcher_count;
        let matchers: Vec<_> = matcher_idxs
            .clone()
            .into_iter()
            .map(|i| format_ident!("Matcher{}", i))
            .collect();

        quote! {
            pub fn active_matcher<'a>(&'a self) -> (#enum_ident, RefMut<'a, fairjax_core::MailBox<T>>) {
                match *self.active_matcher.borrow() {
                    #( #enum_ident::#matchers => (#enum_ident::#matchers, self.mailboxes[#matcher_idxs].borrow_mut()) ),* ,
                }
            }
        }
    }
}
