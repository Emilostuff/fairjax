use crate::utils::{extract_group, parse_identifier, split_by_char, split_by_double_char};
use proc_macro2::{Delimiter, Group, Ident, TokenStream};
use quote::quote;
use syn::Result;

#[derive(Debug, Clone)]
pub enum EnumType {
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

    pub fn to_anonymous_pattern_syntax(&self) -> TokenStream {
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
pub struct SubPattern {
    pub full: TokenStream,
    pub variant_ident: Ident,
    pub full_until_data: TokenStream,
    pub enum_type: EnumType,
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
pub struct Pattern(pub Vec<SubPattern>);

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
}
