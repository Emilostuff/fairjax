pub mod key;

use crate::parse::pattern::PatternDefinition;
use crate::{parse::context::Context, traits::CaseBundle};
use proc_macro2::{Ident, TokenStream};
use quote::quote_spanned;

pub struct PartitionsCompiler;

impl PartitionsCompiler {
    /// Generate declaration code for a BruteForceMatcher
    pub fn generate(
        pattern: &PatternDefinition,
        uniting_vars: &Vec<String>,
        context: Context,
        bundle: &dyn CaseBundle,
        factory_ident: &Ident,
        inner_factory_ident: &Ident,
    ) -> TokenStream {
        // Retrieve values for code block
        let message_type = context.message_type.clone();

        // Generate code for key extraction fn
        let key_fn_ident = Ident::new(
            &format!("key_{}", bundle.case().index()),
            bundle.case().span(),
        );
        let key_code = crate::compile::matchers::partitions::key::KeyExtractionCompiler::generate(
            pattern,
            uniting_vars,
            &key_fn_ident,
            context,
        );

        // Assemble code snippets
        quote_spanned! { bundle.case().span() =>
            #key_code

            let #factory_ident = || {
                Box::new(
                    fairjax_core::strategies::partitions::PartitionsMatcher::<#message_type, _>::new(
                        #inner_factory_ident,
                        #key_fn_ident
                    )
                ) as Box<dyn fairjax_core::CaseHandler<#message_type>>
            };
        }
    }
}
