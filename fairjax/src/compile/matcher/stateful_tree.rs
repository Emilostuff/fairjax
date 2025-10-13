use crate::parse::pattern::{Pattern, SubPattern};
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;

pub struct StatefulTreeMatcherGenerator {
    pattern: Pattern,
    message_type: Ident,
    struct_ident: Ident,
    guard_ident: Ident,
}

impl StatefulTreeMatcherGenerator {
    pub fn new(
        pattern: Pattern,
        message_type: Ident,
        struct_ident: Ident,
        guard_ident: Ident,
    ) -> Self {
        Self {
            pattern,
            message_type,
            struct_ident,
            guard_ident,
        }
    }

    fn generate_match_arm_code(&self) -> TokenStream {
        // Count occurrences of each variant, keep reference to first sub-pattern instance
        let mut variant_info: HashMap<String, (usize, &SubPattern)> = HashMap::new();

        for sub_pattern in &self.pattern.0 {
            let variant_key = sub_pattern.variant_ident.to_string();
            variant_info
                .entry(variant_key)
                .and_modify(|(count, _)| *count += 1)
                .or_insert((1, sub_pattern));
        }

        // Extract variant data and calculate their index ranges, sort output based on Ident
        let mut sorted_pairs: Vec<_> = variant_info.iter().collect();
        sorted_pairs.sort_by_key(|(key, _)| *key);
        let variant_data: Vec<_> = sorted_pairs
            .into_iter()
            .map(|(_, &(count, subpattern))| {
                let enum_type = subpattern.enum_type.to_anonymous_pattern_syntax();
                (count, &subpattern.full_until_data, enum_type)
            })
            .collect();

        // Generate index ranges and match arms code
        let match_arms =
            variant_data
                .into_iter()
                .scan(0, |position, (count, full_ident, enum_type)| {
                    let start = *position;
                    let end = start + count;
                    *position = end;

                    Some(quote! {
                        #full_ident #enum_type => (#start, #end)
                    })
                });

        // Combine everything
        quote!(#(#match_arms),*,)
    }

    fn generate_element_mapping_code(&self) -> TokenStream {
        // Map each variant to all positions where it appears in the pattern
        let mut message_variant_positions: HashMap<String, Vec<usize>> = HashMap::new();
        let mut message_idents: Vec<String> = Vec::new();

        for (index, subpattern) in self.pattern.0.iter().enumerate() {
            message_variant_positions
                .entry(subpattern.variant_ident.to_string())
                .or_default()
                .push(index);

            message_idents.push(subpattern.variant_ident.to_string());
        }

        // Generate Element mapping code for each position in messages list
        message_idents.sort();
        let element_mappings = message_idents.iter().enumerate().map(|(index, ident)| {
            let positions = &message_variant_positions[ident];

            quote!(fairjax_core::stateful_tree::permute::Element::new(self.messages[#index].unwrap(), vec![#(#positions),*]))
        });

        quote!(#(#element_mappings),*)
    }
}

// Codegen
impl StatefulTreeMatcherGenerator {
    pub fn generate_init_code(&self) -> TokenStream {
        let struct_ident = self.struct_ident.clone();
        let guard_ident = self.guard_ident.clone();
        let message_type = self.message_type.clone();
        return quote!(fairjax_core::stateful_tree::StatefulTreeMatcher::<#struct_ident, #message_type>::new(#guard_ident));
    }

    pub fn generate_declaration_code(&self) -> TokenStream {
        let match_arms = self.generate_match_arm_code();
        let pattern_len = self.pattern.0.len();
        let element_mappings = self.generate_element_mapping_code();

        let struct_ident = self.struct_ident.clone();
        let message_type = self.message_type.clone();

        // Assemble
        quote! {
            #[derive(Default, Clone, Debug)]
            pub struct #struct_ident {
                messages: [Option<fairjax_core::MessageId>; #pattern_len],
                counter: usize,
            }

            impl fairjax_core::stateful_tree::PartialMatch<#message_type> for #struct_ident {
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

                fn is_complete(&self) -> bool {
                    self.counter >= #pattern_len
                }

                fn message_ids(&self) -> Vec<fairjax_core::MessageId> {
                    self.messages.iter().filter_map(|x| *x).collect()
                }

                fn to_elements(&self) -> Vec<fairjax_core::stateful_tree::permute::Element> {
                    vec![
                        #element_mappings
                    ]
                }
            }
        }
    }
}

#[cfg(test)]
mod pattern_codegen_tests {
    use super::*;
    use crate::utils::compare_token_streams;
    use proc_macro2::{Ident, Span};

    #[test]
    fn test_generate_declaration_code() {
        let pattern = Pattern::parse(quote!(A(x) && B(z) && A(y))).unwrap();

        let message_type = Ident::new("MyMessage", Span::call_site());
        let struct_name = Ident::new("FairjaxGenerated0", Span::call_site());
        let pattern_gen = StatefulTreeMatcherGenerator::new(
            pattern,
            message_type,
            struct_name,
            Ident::new("guard_fn", Span::call_site()),
        );
        let output = pattern_gen.generate_declaration_code();

        let expected = quote! {
            #[derive(Default, Clone, Debug)]
            pub struct FairjaxGenerated0 {
                messages: [Option<fairjax_core::MessageId>; 3usize],
                counter: usize,
            }

            impl fairjax_core::stateful_tree::PartialMatch<MyMessage> for FairjaxGenerated0 {
                fn extend(&self, message: &MyMessage, id: fairjax_core::MessageId) -> Option<Self> {
                    let mut new_group = self.clone();
                    let (i, j) = match message {
                        A (_) => (0usize, 2usize),
                        B (_) => (2usize, 3usize),
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

                fn is_complete(&self) -> bool {
                    self.counter >= 3usize
                }

                fn message_ids(&self) -> Vec<fairjax_core::MessageId> {
                    self.messages.iter().filter_map(|x| *x).collect()
                }

                fn to_elements(&self) -> Vec<fairjax_core::stateful_tree::permute::Element> {
                    vec![
                        fairjax_core::stateful_tree::permute::Element::new(self.messages[0usize].unwrap(), vec![0usize, 2usize]),
                        fairjax_core::stateful_tree::permute::Element::new(self.messages[1usize].unwrap(), vec![0usize, 2usize]),
                        fairjax_core::stateful_tree::permute::Element::new(self.messages[2usize].unwrap(), vec![1usize])
                    ]
                }
            }
        };

        compare_token_streams(&expected, &output);
    }
}
