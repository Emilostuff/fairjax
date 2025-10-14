use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;

pub struct BruteForceMatcherGenerator {
    message_type: Ident,
    guard_ident: Ident,
    pattern_size: usize,
}

impl BruteForceMatcherGenerator {
    pub fn new(message_type: Ident, guard_ident: Ident, pattern_size: usize) -> Self {
        Self {
            message_type,
            guard_ident,
            pattern_size,
        }
    }

    pub fn generate_init_code(&self) -> TokenStream {
        let guard_ident = self.guard_ident.clone();
        let message_type = self.message_type.clone();
        let pattern_size = self.pattern_size;
        return quote!(fairjax_core::brute_force::BruteForceMatcher::<#message_type>::new(#guard_ident, #pattern_size));
    }

    pub fn generate_declaration_code(&self) -> TokenStream {
        quote!()
    }
}
