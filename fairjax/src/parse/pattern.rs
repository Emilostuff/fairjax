use std::collections::HashMap;

use crate::utils::{extract_group, parse_identifier, split_by_char, split_by_double_char};
use proc_macro2::{Delimiter, Group, Ident, TokenStream};
use quote::quote;
use syn::Result;

#[derive(Debug, Clone)]
enum EnumType {
    Empty,
    Tuple(u8),
    NamedFields,
}

impl EnumType {
    fn parse(group: Group) -> Result<Self> {
        match group.delimiter() {
            Delimiter::Parenthesis => Ok(EnumType::Tuple(
                split_by_char(group.stream(), ',').len() as u8
            )),
            Delimiter::Brace => Ok(EnumType::NamedFields),
            _ => Err(syn::Error::new_spanned(
                group,
                "Enum body must use either () or {}",
            )),
        }
    }

    fn to_anonymous_pattern_syntax(&self) -> TokenStream {
        match self {
            EnumType::Empty => TokenStream::new(),
            EnumType::Tuple(count) => {
                let underscores = (0..*count).map(|_| quote!(_));
                quote!((#(#underscores),*))
            }
            EnumType::NamedFields => quote!({ .. }),
        }
    }
}

#[derive(Debug, Clone)]
struct SubPattern {
    full: TokenStream,
    variant_ident: Ident,
    full_until_data: TokenStream,
    enum_type: EnumType,
}

impl SubPattern {
    pub fn parse(input: TokenStream) -> Result<Self> {
        // Extract variant identifier
        let variant_ident = split_by_double_char(input.clone(), ':')
            .last()
            .ok_or_else(|| {
                syn::Error::new_spanned(input.clone(), "Failed to extract variant identifier")
            })
            .map(|ts| parse_identifier(ts, true))??;

        // Extract grouping if present
        match extract_group(&input) {
            None => Ok(Self {
                full: input.clone(),
                variant_ident,
                full_until_data: input.clone(),
                enum_type: EnumType::Empty,
            }),
            Some(res) if res.postfix.is_empty() => Ok(Self {
                full: input.clone(),
                variant_ident,
                full_until_data: res.prefix,
                enum_type: EnumType::parse(res.group)?,
            }),
            Some(res) => Err(syn::Error::new_spanned(res.postfix, "Unexpected tokens")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pattern(Vec<SubPattern>);

impl Pattern {
    pub fn parse(input: TokenStream) -> Result<Self> {
        let sub_patterns = split_by_double_char(input, '&');
        Ok(Pattern(
            sub_patterns
                .into_iter()
                .map(|p| SubPattern::parse(p))
                .collect::<Result<Vec<SubPattern>>>()?,
        ))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn generate_full_pattern(&self) -> TokenStream {
        let full_sub_patterns = self.0.iter().map(|e| e.full.clone());

        quote! {
            #(#full_sub_patterns),*
        }
    }

    fn generate_match_arm_code(&self) -> TokenStream {
        // Count occurrences of each variant, keep reference to first sub-pattern instance
        let mut variant_info: HashMap<String, (usize, &SubPattern)> = HashMap::new();

        for sub_pattern in &self.0 {
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

        for (index, subpattern) in self.0.iter().enumerate() {
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

            quote!(fairjax_core::permute::Element::new(self.messages[#index].unwrap(), vec![#(#positions),*]))
        });

        quote!(#(#element_mappings),*)
    }

    pub fn generate_declaration_code(
        &self,
        message_type: Ident,
        struct_ident: Ident,
    ) -> TokenStream {
        let match_arms = self.generate_match_arm_code();
        let pattern_len = self.len();
        let element_mappings = self.generate_element_mapping_code();

        // Assemble
        quote! {
            #[derive(Default, Clone, Debug)]
            pub struct #struct_ident {
                messages: [Option<fairjax_core::MessageId>; #pattern_len],
                counter: usize,
            }

            impl fairjax_core::matchgroup::MatchGroup<#message_type> for #struct_ident {
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

                fn to_elements(&self) -> Vec<fairjax_core::permute::Element> {
                    vec![
                        #element_mappings
                    ]
                }
            }
        }
    }
}

//tests

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
        let output = pattern.generate_declaration_code(message_type, struct_name);

        let expected = quote! {
            #[derive(Default, Clone, Debug)]
            pub struct FairjaxGenerated0 {
                messages: [Option<fairjax_core::MessageId>; 3usize],
                counter: usize,
            }

            impl fairjax_core::matchgroup::MatchGroup<MyMessage> for FairjaxGenerated0 {
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

                fn to_elements(&self) -> Vec<fairjax_core::permute::Element> {
                    vec![
                        fairjax_core::permute::Element::new(self.messages[0usize].unwrap(), vec![0usize, 2usize]),
                        fairjax_core::permute::Element::new(self.messages[1usize].unwrap(), vec![0usize, 2usize]),
                        fairjax_core::permute::Element::new(self.messages[2usize].unwrap(), vec![1usize])
                    ]
                }
            }
        };

        compare_token_streams(&expected, &output);
    }
}
