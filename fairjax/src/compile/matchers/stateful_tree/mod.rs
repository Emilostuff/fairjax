pub mod element_mapping;
pub mod match_arms;
pub mod profile;

use element_mapping::ElementMappingCodeGen;
use match_arms::MatchArmCodeGen;
use proc_macro2::TokenStream;

use crate::compile::case::guard::GuardCodeGen;
use crate::compile::pattern::full::PatternCompiler;
use crate::compile::pattern::sub::SubPatternCompiler;
use crate::parse::{case::Case, context::Context};
use profile::PatternProfile;
use quote::{format_ident, quote};
use syn::{Ident, Path};

pub struct StatefulTreeCompiler;

impl StatefulTreeCompiler {
    pub fn generate<G: GuardCodeGen, M: MatchArmCodeGen, E: ElementMappingCodeGen>(
        case: &dyn Case,
        context: Context,
        output_ident: &Ident,
    ) -> TokenStream {
        // Profile pattern
        let profile = PatternProfile::new(case.pattern());

        // Retrieve values for code block
        let message_type = &context.message_type;
        let pattern_size = case.pattern().len();

        // Guard declaration code
        let guard_ident = format_ident!("fairjax_st_guard_function_{}", case.index());
        let guard_code = G::generate::<PatternCompiler>(case, &context, &guard_ident);

        // Struct declaration code
        let struct_ident = format_ident!("FairjaxGeneratedStatefulTreeNodeData{}", case.index());
        let struct_code = Self::gen_struct_declaration_code(&struct_ident, pattern_size);

        // Struct methods declrataion code
        let extent_method_code = Self::gen_extend_code::<M>(&profile, &message_type);
        let is_complete_code = Self::gen_is_complete_code(pattern_size);
        let message_ids_method_code = Self::gen_message_ids_code(pattern_size);
        let to_element_method_code = Self::gen_to_elements_code::<E>(&profile, pattern_size);

        // Assemble
        quote! {
            #guard_code

            #struct_code

            impl fairjax_core::strategies::stateful_tree::PartialMatch<#pattern_size, #message_type> for #struct_ident {
                #extent_method_code

                #is_complete_code

                #message_ids_method_code

                #to_element_method_code
            }

            let #output_ident = fairjax_core::strategies::stateful_tree::StatefulTreeMatcher::<#pattern_size, #struct_ident, #message_type>::new(#guard_ident);
        }
    }
}

impl StatefulTreeCompiler {
    fn gen_struct_declaration_code(struct_ident: &Ident, pattern_size: usize) -> TokenStream {
        quote! {
            #[derive(Default, Clone, Debug)]
            pub struct #struct_ident {
                messages: [Option<fairjax_core::MessageId>; #pattern_size],
                counter: usize,
            }
        }
    }

    fn gen_extend_code<M: MatchArmCodeGen>(
        profile: &PatternProfile,
        message_type: &Path,
    ) -> TokenStream {
        // Generate match arm code
        let match_arms = M::generate::<SubPatternCompiler>(profile);

        quote! {
            fn extend(&self, message: &#message_type, id: fairjax_core::MessageId) -> Option<Self> {
                let mut new_group = self.clone();
                let (i, j) = match message {
                    #match_arms
                    _ => return None
                };

                for slot in &mut new_group.messages[i..j] {
                    if slot.is_none() {
                        *slot = Some(id);
                        new_group.counter += 1;
                        return Some(new_group);
                    }
                }
                None
            }
        }
    }

    fn gen_is_complete_code(pattern_size: usize) -> TokenStream {
        quote! {
            fn is_complete(&self) -> bool {
                self.counter >= #pattern_size
            }
        }
    }

    fn gen_message_ids_code(pattern_size: usize) -> TokenStream {
        quote! {
            fn message_ids(&self) -> &[Option<fairjax_core::MessageId>; #pattern_size] {
                &self.messages
            }
        }
    }

    fn gen_to_elements_code<E: ElementMappingCodeGen>(
        profile: &PatternProfile,
        pattern_size: usize,
    ) -> TokenStream {
        let element_mappings = E::generate(profile);

        quote! {
            fn to_elements(&self) -> [fairjax_core::strategies::stateful_tree::permute::Element; #pattern_size] {
                #element_mappings
            }
        }
    }
}
