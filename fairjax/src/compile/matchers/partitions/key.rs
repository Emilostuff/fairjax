use proc_macro2::TokenStream;
use quote::{ToTokens, quote_spanned};
use syn::Ident;

use crate::parse::{context::Context, pattern::PatternDefinition};
use crate::traits::Pattern;

pub struct KeyExtractionCompiler;

impl KeyExtractionCompiler {
    pub fn generate(
        pattern: &PatternDefinition,
        partition_vars: &Vec<String>,
        key_fn_ident: &Ident,
        context: Context,
    ) -> TokenStream {
        let sub_patterns = pattern
            .sub_patterns()
            .into_iter()
            .map(|sp| sp.get().to_pattern().to_token_stream())
            .collect::<Vec<_>>();

        let partition_var_idents = partition_vars
            .iter()
            .map(|var| Ident::new(var, pattern.span()))
            .collect::<Vec<_>>();

        let key = quote_spanned!(pattern.span() => (#(#partition_var_idents.clone()),*));
        let message_type = context.message_type;

        quote_spanned! {
            pattern.span() =>
            fn #key_fn_ident(message: &#message_type) -> Option<fairjax_core::AnyKeyBox> {
                #[allow(unused_variables, unreachable_patterns)]
                match message {
                    #(#sub_patterns => Some(fairjax_core::AnyKeyBox::new(#key)),)*
                    _ => None,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro_utils::assert_tokens;
    use proc_macro2::Span;
    use syn::{Pat, parse_quote};

    #[test]
    fn test_generate_key_extraction_compiler() {
        // Define inputs
        let match_arm_pattern: Pat = parse_quote!((
            A(id, 42, id2),
            B {
                a,
                b: id,
                c: "hello",
                d: id2
            }
        ));
        let pattern = PatternDefinition::parse(match_arm_pattern).unwrap();
        let partition_vars = vec!["id".to_string(), "id2".to_string()];
        let key_fn_ident = Ident::new("fn_ident", Span::call_site());
        let context = Context {
            incoming_message: parse_quote!(unused),
            mailbox: parse_quote!(unused),
            message_type: parse_quote!(MsgType),
        };

        // Generate KeyCode
        let key_code =
            KeyExtractionCompiler::generate(&pattern, &partition_vars, &key_fn_ident, context);

        #[rustfmt::skip]
        assert_tokens!(key_code, {
            fn fn_ident(message: &MsgType) -> Option<fairjax_core::AnyKeyBox> {
                #[allow(unused_variables, unreachable_patterns)]
                match message {
                    A(id, 42, id2) => Some( fairjax_core::AnyKeyBox::new(( id.clone(), id2.clone() )) ),
                    B { a, b: id, c: "hello" , d: id2} => Some( fairjax_core::AnyKeyBox::new(( id.clone(), id2.clone())) ),
                    _ => None,
                }
            }
        });
    }
}
