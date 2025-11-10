use crate::parse::sub_pattern::SubPatternDefinition;
use crate::traits::SubPattern;
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{Pat, PatRest, punctuated::Punctuated, spanned::Spanned, token::DotDot};

pub trait SubPatternCodeGen {
    fn generate<'a>(sub_pattern: &'a dyn SubPattern, anonymous: bool) -> TokenStream;
}

pub struct SubPatternCompiler;

impl SubPatternCodeGen for SubPatternCompiler {
    fn generate(sub_pattern: &dyn SubPattern, anonymous: bool) -> TokenStream {
        match sub_pattern.get() {
            SubPatternDefinition::Ident(ident) => quote_spanned!(ident.span() => #ident),
            SubPatternDefinition::Path(p) => quote_spanned!(p.span() => #p),
            SubPatternDefinition::TupleStruct(mut ts) => {
                if anonymous {
                    ts.elems = Punctuated::from_iter(ts.elems.iter().map(|_| {
                        Pat::Wild(syn::PatWild {
                            attrs: vec![],
                            underscore_token: syn::token::Underscore::default(),
                        })
                    }));
                }
                quote_spanned!(ts.span() => #ts)
            }
            SubPatternDefinition::Struct(mut s) => {
                if anonymous {
                    s.fields = Punctuated::new();
                    s.rest = Some(PatRest {
                        attrs: vec![],
                        dot2_token: DotDot::default(),
                    });
                }
                quote_spanned!(s.span() => #s)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro_utils::assert_tokens;
    use quote::format_ident;
    use syn::{
        FieldPat, Pat, PatIdent, PatPath, PatStruct, PatTupleStruct, Path, parse_quote,
        punctuated::Punctuated, token,
    };

    fn pat_ident(name: &str) -> PatIdent {
        PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: format_ident!("{}", name),
            subpat: None,
        }
    }

    fn pat_tuple_struct(path: Path, fields: Vec<&str>) -> PatTupleStruct {
        PatTupleStruct {
            attrs: vec![],
            qself: None,
            path,
            paren_token: token::Paren::default(),
            elems: Punctuated::from_iter(
                fields.into_iter().map(|name| Pat::Ident(pat_ident(name))),
            ),
        }
    }

    fn pat_struct(path: Path, fields: Vec<&str>, rest: bool) -> PatStruct {
        PatStruct {
            attrs: vec![],
            qself: None,
            path,
            brace_token: token::Brace::default(),
            fields: Punctuated::from_iter(fields.into_iter().map(|name| FieldPat {
                attrs: vec![],
                member: syn::Member::Named(format_ident!("{}", name)),
                colon_token: None,
                pat: Box::new(Pat::Ident(pat_ident(name))),
            })),
            rest: rest.then(|| PatRest {
                attrs: vec![],
                dot2_token: DotDot::default(),
            }),
        }
    }

    #[test]
    fn test_generate_path() {
        let pat_path: PatPath = parse_quote!(Foo);
        let sub_pattern = SubPatternDefinition::Path(pat_path);
        let tokens = SubPatternCompiler::generate(&sub_pattern, false);
        assert_tokens!(tokens, { Foo });
    }

    #[test]
    fn test_generate_tuple_struct() {
        let pat_tuple_struct = pat_tuple_struct(parse_quote!(Bar), vec!["a", "b"]);
        let sub_pattern = SubPatternDefinition::TupleStruct(pat_tuple_struct);

        let tokens = SubPatternCompiler::generate(&sub_pattern, false);
        assert_tokens!(tokens, { Bar(a, b) });
    }

    #[test]
    fn test_generate_tuple_struct_anonymus() {
        let pat_tuple_struct = pat_tuple_struct(parse_quote!(Bar), vec!["a", "b"]);
        let sub_pattern = SubPatternDefinition::TupleStruct(pat_tuple_struct);

        let tokens = SubPatternCompiler::generate(&sub_pattern, true);
        assert_tokens!(tokens, { Bar(_, _) });
    }

    #[test]
    fn test_generate_struct() {
        let pat_tuple_struct = pat_struct(parse_quote!(Foo), vec!["a", "b"], false);
        let sub_pattern = SubPatternDefinition::Struct(pat_tuple_struct);

        let tokens = SubPatternCompiler::generate(&sub_pattern, false);
        assert_tokens!(tokens, { Foo { a, b } });
    }

    #[test]
    fn test_generate_struct_partial_pattern() {
        let pat_tuple_struct = pat_struct(parse_quote!(Foo), vec!["a"], true);
        let sub_pattern = SubPatternDefinition::Struct(pat_tuple_struct);

        let tokens = SubPatternCompiler::generate(&sub_pattern, false);
        assert_tokens!(tokens, { Foo { a, .. } });
    }

    #[test]
    fn test_generate_struct_anonymus() {
        let pat_tuple_struct = pat_struct(parse_quote!(Foo), vec!["a", "b"], false);
        let sub_pattern = SubPatternDefinition::Struct(pat_tuple_struct);

        let tokens = SubPatternCompiler::generate(&sub_pattern, true);
        assert_tokens!(tokens, { Foo { .. } });
    }

    #[test]
    fn test_generate_struct_anonymus_partial_pattern() {
        let pat_tuple_struct = pat_struct(parse_quote!(Foo), vec!["a"], true);
        let sub_pattern = SubPatternDefinition::Struct(pat_tuple_struct);

        let tokens = SubPatternCompiler::generate(&sub_pattern, true);
        assert_tokens!(tokens, { Foo { .. } });
    }
}
