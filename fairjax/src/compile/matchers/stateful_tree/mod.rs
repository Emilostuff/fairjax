pub mod mappings;
pub mod match_arms;

use crate::compile::case::accept::AcceptCodeGen;
use crate::compile::matchers::stateful_tree::mappings::MappingCodeGen;
use crate::compile::pattern::sub::SubPatternCompiler;
use crate::parse::context::Context;
use crate::traits::CaseBundle;
use match_arms::MatchArmCodeGen;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{Ident, Path};

pub struct StatefulTreeCompiler;

impl StatefulTreeCompiler {
    pub fn generate<A: AcceptCodeGen, M: MatchArmCodeGen, MP: MappingCodeGen>(
        bundle: &dyn CaseBundle,
        context: Context,
        factory_ident: &Ident,
        guard_fn_ident: &Ident,
    ) -> TokenStream {
        let case = bundle.case();
        let span = case.span();

        // Retrieve values for code block
        let message_type = &context.message_type;
        let pattern_size = case.pattern().len();

        // Accept declaration code
        let accept_fn_name = format!("fairjax_accept_function_{}", case.index());
        let accept_fn_ident = Ident::new(&accept_fn_name, case.span());
        let accept_code = A::generate::<SubPatternCompiler>(case, &context, &accept_fn_name);

        // Mappings declaration code
        let mappings_name = format!("FAIRJAX_MAPPINGS_{}", case.index());
        let mappings_ident = Ident::new(&mappings_name, case.span());
        let mappings_code = MP::generate(span, bundle, &mappings_ident);

        // Struct declaration code
        let struct_ident = Ident::new(
            &format!("FairjaxGeneratedStatefulTreeNodeData{}", case.index()),
            case.span(),
        );
        let struct_code = Self::gen_struct_declaration_code(span, &struct_ident, pattern_size);

        // Struct methods declarataion code
        let extent_method_code = Self::gen_extend_code::<M>(span, bundle, &message_type);
        let is_complete_code = Self::gen_is_complete_code(span, pattern_size);
        let message_ids_method_code = Self::gen_message_ids_code(span, pattern_size);

        // Assemble
        quote_spanned! { span =>
            #accept_code

            #mappings_code

            #struct_code

            impl fairjax_core::strategies::stateful_tree::PartialMatch<#pattern_size, #message_type> for #struct_ident {
                #extent_method_code

                #is_complete_code

                #message_ids_method_code
            }

            let #factory_ident = || {
                Box::new(
                    fairjax_core::strategies::stateful_tree::StatefulTreeMatcher::<
                        #pattern_size,
                        #struct_ident,
                        #message_type
                    >::new(
                        #guard_fn_ident, #accept_fn_ident, &#mappings_ident
                    )
                ) as Box<dyn fairjax_core::CaseHandler<#message_type>>
            };
        }
    }
}

impl StatefulTreeCompiler {
    fn gen_struct_declaration_code(
        span: Span,
        struct_ident: &Ident,
        pattern_size: usize,
    ) -> TokenStream {
        quote_spanned! { span =>
            #[derive(Default, Clone, Debug)]
            pub struct #struct_ident {
                messages: [Option<fairjax_core::MessageId>; #pattern_size],
                counter: usize,
            }
        }
    }

    fn gen_extend_code<M: MatchArmCodeGen>(
        span: Span,
        bundle: &dyn CaseBundle,
        message_type: &Path,
    ) -> TokenStream {
        // Generate match arm code
        let match_arms = M::generate::<SubPatternCompiler>(span, bundle);

        quote_spanned! { span =>
            #[inline(always)]
            fn extend(&self, message: &#message_type, id: fairjax_core::MessageId) -> Option<Self> {
                let mut new_group = self.clone();
                #[allow(unreachable_patterns)]
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

    fn gen_is_complete_code(span: Span, pattern_size: usize) -> TokenStream {
        quote_spanned! { span =>
            #[inline(always)]
            fn is_complete(&self) -> bool {
                self.counter >= #pattern_size
            }
        }
    }

    fn gen_message_ids_code(span: Span, pattern_size: usize) -> TokenStream {
        quote_spanned! { span =>
            #[inline(always)]
            fn message_ids(&self) -> &[Option<fairjax_core::MessageId>; #pattern_size] {
                &self.messages
            }
        }
    }
}
