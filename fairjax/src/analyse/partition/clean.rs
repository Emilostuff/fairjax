use crate::parse::sub_pattern::SubPatternDefinition;
use quote::ToTokens;
use syn::{FieldPat, Pat, PatRest, Token, punctuated::Punctuated, token::DotDot};

/// Cleans sub-patterns by removing or replacing occurrences of a partition variable.
pub struct SubPatternCleaner;

impl SubPatternCleaner {
    /// Entry point: cleans a sub-pattern by removing partition_var occurrences.
    pub fn clean(sub_pattern: &mut SubPatternDefinition, partition_vars: &Vec<String>) {
        // We do not allow cleaning of Idents on Sub-Pattern level, as they turn into Wildcards
        if matches!(sub_pattern, SubPatternDefinition::Ident(_)) {
            unreachable!(
                "Trying to clean Sub-Pattern: {} of type Ident, which must never happen",
                sub_pattern.to_pattern().to_token_stream().to_string(),
            );
        }

        // Otherwise clean Sub-Pattern
        *sub_pattern = match Self::clean_rec(sub_pattern.to_pattern(), partition_vars) {
            Pat::Path(path) => SubPatternDefinition::Path(path),
            Pat::TupleStruct(tuple_struct) => SubPatternDefinition::TupleStruct(tuple_struct),
            Pat::Struct(struct_pat) => SubPatternDefinition::Struct(struct_pat),
            _ => unreachable!(),
        };
    }

    /// Recursively cleans patterns, replacing partition_var with a wildcard.
    fn clean_rec(pat: Pat, partition_vars: &Vec<String>) -> Pat {
        use Pat::*;
        return match pat {
            // If the ident matches the partition_var, replace with wildcard
            Ident(ident) if partition_vars.contains(&ident.ident.to_string()) => {
                Self::get_wildcard()
            }
            // Leave ident untouched if it doesn't match
            Ident(ident) => Ident(ident),
            // Process the internal pattern of the parenthesis
            Paren(mut pat_paren) => {
                pat_paren.pat = Box::new(Self::clean_rec(*pat_paren.pat, partition_vars));
                Paren(pat_paren)
            }
            // Process each element in the slice
            Slice(mut pat_slice) => {
                pat_slice.elems = Self::clean_elems(pat_slice.elems, partition_vars);
                Slice(pat_slice)
            }
            // Process each element in the tuple
            Tuple(mut pat_tuple) => {
                pat_tuple.elems = Self::clean_elems(pat_tuple.elems, partition_vars);
                Tuple(pat_tuple)
            }
            // Process each element in the tuple struct
            TupleStruct(mut pat_tuple_struct) => {
                pat_tuple_struct.elems = Self::clean_elems(pat_tuple_struct.elems, partition_vars);
                TupleStruct(pat_tuple_struct)
            }
            // Process each field in the struct
            Struct(mut pat_struct) => {
                let initial_field_count = pat_struct.fields.len();
                pat_struct.fields = Self::clean_fields(pat_struct.fields, partition_vars);

                // Add PatRest to the struct if any fields were removed
                if pat_struct.fields.len() != initial_field_count {
                    pat_struct.rest = Self::get_rest();
                }

                Struct(pat_struct)
            }
            // Other patterns: return as is (no cleaning needed, based on what we allow in the count module)
            _ => pat,
        };
    }

    /// Returns a wildcard pattern (_).
    fn get_wildcard() -> Pat {
        Pat::Wild(syn::PatWild {
            attrs: vec![],
            underscore_token: syn::token::Underscore::default(),
        })
    }

    /// Returns a rest pattern ([..]).
    fn get_rest() -> Option<PatRest> {
        Some(PatRest {
            attrs: vec![],
            dot2_token: DotDot::default(),
        })
    }

    /// Cleans each element in a punctuated list of patterns.
    fn clean_elems(
        elems: Punctuated<Pat, Token![,]>,
        partition_vars: &Vec<String>,
    ) -> Punctuated<Pat, Token![,]> {
        let cleaned = elems
            .into_iter()
            .map(|pat| Self::clean_rec(pat, partition_vars))
            .collect::<Vec<_>>();
        Punctuated::from_iter(cleaned)
    }

    /// Cleans each field in a punctuated list of Field Patterns.
    /// Removes the field if its a top-level ident matching partiton_var
    fn clean_fields(
        fields: Punctuated<FieldPat, Token![,]>,
        partition_vars: &Vec<String>,
    ) -> Punctuated<FieldPat, Token![,]> {
        use Pat::*;
        let cleaned = fields
            .into_iter()
            .filter_map(|mut field| match *field.pat {
                // Remove field if identifier matches partition_var
                Ident(ident) if partition_vars.contains(&ident.ident.to_string()) => None,
                // Keep field unchanged if identifier does not match
                Ident(_) => Some(field),
                // Recursively clean other field patterns
                _ => {
                    field.pat = Box::new(Self::clean_rec(*field.pat, partition_vars));
                    Some(field)
                }
            })
            .collect::<Vec<_>>();

        Punctuated::from_iter(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro_utils::assert_tokens;
    use syn::{Pat, parse_quote};

    #[test]
    fn test_clean_rec_ident_match() {
        // Pattern definition
        let pat: Pat = parse_quote! { id };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness
        assert_tokens!(result.to_token_stream(), { _ });
    }

    #[test]
    fn test_clean_rec_ident_no_match() {
        // Pattern definition
        let pat: Pat = parse_quote! { foo };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness
        assert_tokens!(result.to_token_stream(), { foo });
    }

    #[test]
    fn test_clean_rec_paren() {
        // Pattern definition
        let pat: Pat = parse_quote! { (id) };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness
        assert_tokens!(result.to_token_stream(), { (_) });
    }

    #[test]
    fn test_clean_rec_slice() {
        // Pattern definition
        let pat: Pat = parse_quote! { [a, b, id, c, d] };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness
        assert_tokens!(result.to_token_stream(), { [a, b, _, c, d] });
    }

    #[test]
    fn test_clean_rec_tuple() {
        // Pattern definition
        let pat: Pat = parse_quote! { (a, b, id, c, d) };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness
        assert_tokens!(result.to_token_stream(), { (a, b, _, c, d) });
    }

    #[test]
    fn test_clean_rec_tuple_struct() {
        // Pattern definition
        let pat: Pat = parse_quote! { MyTupleStruct(a, b, id, c, d) };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness
        assert_tokens!(result.to_token_stream(), { MyTupleStruct(a, b, _, c, d) });
    }

    #[test]
    fn test_clean_rec_struct_match() {
        // Pattern definition
        let pat: Pat = parse_quote! { MyStruct{ a, b, id, c, d } };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness
        assert_tokens!(result.to_token_stream(), { MyStruct { a, b, c, d, .. } });
    }

    #[test]
    fn test_clean_rec_struct_no_match() {
        // Pattern definition
        let pat: Pat = parse_quote! { MyStruct{ a, b, foo, c, d } };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness
        assert_tokens!(result.to_token_stream(), { MyStruct { a, b, foo, c, d } });
    }

    #[test]
    fn test_clean_rec_path() {
        // Pattern definition
        let pat: Pat = parse_quote! { MyType::id };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness: Path patterns should not be cleaned
        assert_tokens!(result.to_token_stream(), { MyType::id });
    }

    #[test]
    fn test_clean_rec_struct() {
        // Pattern definition
        let pat: Pat = parse_quote! { MyStruct { foo, id, bar } };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness:
        assert_tokens!(result.to_token_stream(), { MyStruct { foo, bar, .. } });
    }

    #[test]
    fn test_clean_rec_nested_tuple_struct_in_struct() {
        // Pattern definition
        let pat: Pat = parse_quote! { MyStruct { a, b, inner: MyTupleStruct(x, id, y), c } };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness
        #[rustfmt::skip] // Prevent Rustfmt from formatting and adding commas
        assert_tokens!(result.to_token_stream(), {
            MyStruct {a, b, inner: MyTupleStruct(x, _, y), c }
        });
    }

    #[test]
    fn test_clean_rec_nested_struct_in_tuple() {
        // Pattern definition
        let pat: Pat = parse_quote! { (foo, Bar { id, x, y }, baz) };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness
        #[rustfmt::skip] // Prevent Rustfmt from formatting and adding commas
        assert_tokens!(result.to_token_stream(), { (foo, Bar { x, y, .. }, baz) });
    }

    #[test]
    fn test_clean_rec_deeply_nested_patterns() {
        // Pattern definition
        let pat: Pat = parse_quote! { Outer { a, inner: (X { id, b }, [id, y, z]), d } };
        let partition_vars = vec!["id".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness
        #[rustfmt::skip] // Prevent Rustfmt from formatting and adding commas
        assert_tokens!(result.to_token_stream(), {
            Outer {a, inner: (X { b, .. }, [_, y, z]), d }
        });
    }

    #[test]
    fn test_clean_rec_nested_multiple_partition_vars() {
        // Pattern definition with multiple partition vars
        let pat: Pat = syn::parse_quote! {
            Outer { a, inner: ( X(id, b, c), [d, e, id2, f]), g, id3 }
        };
        let partition_vars = vec!["id".to_string(), "id2".to_string(), "id3".to_string()];

        // Clean pattern
        let result = SubPatternCleaner::clean_rec(pat, &partition_vars);

        // Assert Correctness
        #[rustfmt::skip]
        assert_tokens!(result.to_token_stream(), {
            Outer { a, inner: ( X(_, b, c), [d, e, _, f]), g, .. }
        });
    }

    #[test]
    #[should_panic]
    fn test_clean_ident() {
        // Define input pattern
        let mut sub_pattern = SubPatternDefinition::parse(parse_quote! { id }).unwrap();
        let partition_vars = vec!["id".to_string()];

        // Clean pattern - should panic as we don't allow cleaning standalone Idents
        SubPatternCleaner::clean(&mut sub_pattern, &partition_vars);
    }

    #[test]
    fn test_clean_sub_pattern() {
        // Define input pattern
        let mut sub_pattern =
            SubPatternDefinition::parse(parse_quote! { A(id, x, y, id2) }).unwrap();
        let partition_vars = vec!["id".to_string(), "id2".to_string()];

        // Clean pattern - should panic as we don't allow cleaning standalone Idents
        SubPatternCleaner::clean(&mut sub_pattern, &partition_vars);

        assert_tokens!(sub_pattern.to_pattern().to_token_stream(), {
            A(_, x, y, _)
        });
    }
}
