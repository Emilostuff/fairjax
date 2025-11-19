use crate::{parse::pattern::PatternDefinition, traits::Pattern};
use proc_macro2::Span;
use std::collections::BTreeMap;
use syn::{Ident, Pat, PatSlice, PatTuple, PatTupleStruct, Result};

/// Stores all spans where an Ident appears in a sub-pattern.
#[derive(Debug, Clone)]
pub struct SubPatternOccurrences(pub Vec<Span>);

impl SubPatternOccurrences {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Tracks all Ident occurrences across all sub-patterns in a Pattern.
#[derive(Clone)]
pub struct IdentCounts {
    pub(super) pattern_size: usize,
    pub(super) counts: BTreeMap<String, Vec<SubPatternOccurrences>>,
}

impl IdentCounts {
    /// Analyse a pattern to return the IdentCounts, i.e. all Ident occurrences across all sub-patterns
    pub fn analyse(pat_def: &PatternDefinition) -> Result<Self> {
        let mut acc = Self {
            pattern_size: pat_def.len(),
            counts: BTreeMap::new(),
        };
        for (i, sub_pattern) in pat_def.sub_patterns.iter().enumerate() {
            acc.count_rec(&sub_pattern.to_syn_pattern(), i)?;
        }
        Ok(acc)
    }

    /// Add an Ident occurence
    fn add(&mut self, ident: &Ident, index: usize) {
        self.counts
            .entry(ident.to_string())
            .or_insert(vec![SubPatternOccurrences(vec![]); self.pattern_size])[index]
            .0
            .push(ident.span());
    }

    /// Recursively count all Ident occurences in a sub-pattern and add them to the internal data structure
    fn count_rec(&mut self, sub_pattern: &Pat, index: usize) -> Result<()> {
        use Pat::*;
        return match sub_pattern {
            // We allow all these types in the counter without processing, although some (like Or)
            // are not supported. The compiler handles use of unsupported types down the road.
            Const(_) | Lit(_) | Or(_) | Path(_) | Range(_) | Reference(_) | Rest(_) | Wild(_)
            | Type(_) => Ok(()),
            // Count occurences in all supported types
            Ident(ident) => Ok({
                self.add(&ident.ident, index);
                if let Some((_, sub_pat)) = &ident.subpat {
                    self.count_rec(&sub_pat, index)?;
                }
            }),
            Paren(pat_paren) => self.count_rec(&pat_paren.pat, index),
            Slice(PatSlice { elems, .. })
            | Tuple(PatTuple { elems, .. })
            | TupleStruct(PatTupleStruct { elems, .. }) => {
                elems.iter().map(|pat| self.count_rec(pat, index)).collect()
            }
            Struct(pat_struct) => pat_struct
                .fields
                .iter()
                .map(|field| self.count_rec(&field.pat, index))
                .collect(),
            // Handle use of unsopprted inputs in the Macro
            Macro(pat_macro) => Err(syn::Error::new_spanned(
                pat_macro,
                "Macro invokations in message pattern are not allowed",
            )),
            Verbatim(tokens) => Err(syn::Error::new_spanned(
                tokens,
                "Tokenstreams in message pattern are not allowed",
            )),
            &_ => Err(syn::Error::new_spanned(
                sub_pattern,
                "Unknown pattern in message pattern.",
            )),
        };
    }
}

/// Make the BTreeMap available as an iterator for later processing
impl IntoIterator for IdentCounts {
    type Item = (String, Vec<SubPatternOccurrences>);
    type IntoIter = std::collections::btree_map::IntoIter<String, Vec<SubPatternOccurrences>>;

    fn into_iter(self) -> Self::IntoIter {
        self.counts.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Pat, parse_quote};

    /// Create an IdentCounts object and count occurences in the provided Pattern recursively.
    /// Used for testing the `count_rec` method only!
    fn create_and_count_rec(pat: Pat) -> IdentCounts {
        let mut counts = IdentCounts {
            pattern_size: 1,
            counts: BTreeMap::new(),
        };
        counts.count_rec(&pat, 0).unwrap();
        counts
    }

    /// Asserts that an IdentCounts object contains the specified number of occurrences for each ident
    /// across all subpatterns.
    /// This flattens the structure and is only useful when evaluating `count_rec`.
    ///
    /// # Parameters
    /// - `expected_ident_occurences`: A vector of tuples where each tuple is `(expected_count, ident_name)`.
    ///   `expected_count` is the total number of times `ident_name` should appear across all subpatterns.
    /// - `result`: The IdentCounts object to check, containing the counted identifier occurrences.
    fn assert_counts(expected_ident_occurences: Vec<(usize, &str)>, result: IdentCounts) {
        for (expected_count, ident) in expected_ident_occurences.iter() {
            let actual_count = result
                .counts
                .get(&ident.to_string())
                .expect(&format!("'{}' must be in IdentCouts object", ident))
                .iter()
                .map(|occ| occ.len())
                .sum::<usize>();

            assert_eq!(
                *expected_count, actual_count,
                "\nFailed at Ident: {}",
                ident
            );
        }
    }

    /// Asserts that an IdentCounts object contains the specified number of occurrences for each ident
    /// in each sub-pattern, as specified by the expected positions vector.
    ///
    /// # Parameters
    /// - `expected_ident_counts`: A vector of tuples where each tuple is `(expected_counts, ident_name)`.
    ///   `expected_counts` is a vector of expected counts for each sub-pattern index for the given identifier.
    /// - `result`: The IdentCounts object to check, containing the counted identifier occurrences.
    fn assert_positions(expected_ident_counts: Vec<(Vec<usize>, &str)>, result: IdentCounts) {
        for (expected_counts, ident) in expected_ident_counts.iter() {
            let occurrences = result
                .counts
                .get(&ident.to_string())
                .expect(&format!("'{}' must be in IdentCounts object", ident));

            for (i, (actual, expected)) in occurrences.iter().zip(expected_counts).enumerate() {
                assert_eq!(
                    *expected,
                    actual.len(),
                    "\nFailed at Ident: '{}'\nSub-Pattern: {}",
                    ident,
                    i
                );
            }
        }
    }

    /// Checks that none of the provided idents are present in the IdentCounts object
    fn assert_none(idents: Vec<&str>, result: IdentCounts) {
        for ident in idents {
            assert!(result.counts.get(&ident.to_string()).is_none());
        }
    }

    #[test]
    fn test_count_rec_simple_ident() {
        // Pattern definition
        let pat: Pat = parse_quote! { foo };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness
        assert_counts(vec![(1, "foo")], result);
    }

    #[test]
    fn test_count_rec_tuple() {
        // Pattern definition
        let pat: Pat = parse_quote! { (foo, bar, baz) };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness
        assert_counts(vec![(1, "foo"), (1, "bar"), (1, "baz")], result);
    }

    #[test]
    fn test_count_rec_nested_tuple() {
        // Pattern definition
        let pat: Pat = parse_quote! { (foo, (bar, baz), foo) };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness
        assert_counts(vec![(2, "foo"), (1, "bar"), (1, "baz")], result);
    }

    #[test]
    fn test_count_rec_struct() {
        // Pattern definition
        let pat: Pat = parse_quote! { MyStruct { a, b, c } };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness
        assert_counts(vec![(1, "a"), (1, "b"), (1, "c")], result);
    }

    #[test]
    fn test_count_rec_struct_with_repeated_ident() {
        // Pattern definition
        let pat: Pat = parse_quote! { MyStruct { a, b, a } };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness
        assert_counts(vec![(2, "a"), (1, "b")], result);
    }

    #[test]
    fn test_count_rec_slice() {
        // Pattern definition
        let pat: Pat = parse_quote! { [foo, bar, foo] };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness
        assert_counts(vec![(2, "foo"), (1, "bar")], result);
    }

    #[test]
    fn test_count_rec_tuple_struct() {
        // Pattern definition
        let pat: Pat = parse_quote! { MyTupleStruct(foo, bar, foo) };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness
        assert_counts(vec![(2, "foo"), (1, "bar")], result);
    }

    #[test]
    fn test_count_rec_enum_variant() {
        // Pattern definition
        let pat: Pat = parse_quote! { MyTupleStruct(MyEnumVariant, a) };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness
        assert_counts(vec![(1, "MyEnumVariant"), (1, "a")], result);
    }

    #[test]
    fn test_count_rec_wild_and_const() {
        // Pattern definition
        let pat: Pat = parse_quote! { (_, 42, foo) };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness - Wildcards are ignored
        assert_counts(vec![(1, "foo")], result.clone());
        assert_none(vec!["_"], result);
    }

    #[test]
    fn test_count_rec_or() {
        // Pattern definition
        let pat: Pat = parse_quote! { foo | bar | baz };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness - Or patterns are ignored
        assert_none(vec!["foo", "bar", "baz"], result);
    }

    #[test]
    fn test_count_rec_reference() {
        // Pattern definition
        let pat: Pat = parse_quote! { &foo };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness - Reference patterns are ignored
        assert_none(vec!["foo"], result);
    }

    #[test]
    fn test_count_rec_range() {
        // Pattern definition
        let pat: Pat = parse_quote! { foo..bar };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness - Range patterns are ignored
        assert_none(vec!["foo", "bar"], result);
    }

    #[test]
    #[should_panic]
    fn test_count_rec_macro() {
        // Pattern definition
        let pat: Pat = parse_quote! { my_macro!(foo) };

        // Count occurences in pattern - Should panic
        create_and_count_rec(pat);
    }

    #[test]
    fn test_count_rec_deeper_nesting() {
        // Pattern definition:
        let pat: Pat =
            parse_quote! { (((foo, [MyStruct { x: (MyTupleStruct(bar, _), _, _)}, ..], baz)), _) };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness
        assert_counts(vec![(1, "foo"), (1, "bar"), (1, "baz")], result);
    }

    #[test]
    fn test_count_rec_big_nested_pat() {
        // Pattern definition
        let pat: Pat = parse_quote! {
            MyStruct {
                a,
                b: MyTupleStruct(x, [y, z, MyStruct { a: w, b: _, c: v }]),
                c: (u, (t, s), [r, q]),
                d: (p, (o)),
                e: [x, y, (z, w)],
                f: MyTupleStruct(n, [v, u], (t, s)),
            }
        };

        // Count occurences in pattern
        let result = create_and_count_rec(pat);

        // Assert correctness
        assert_counts(
            vec![
                (1, "a"),
                (2, "x"),
                (2, "y"),
                (2, "z"),
                (2, "w"),
                (2, "v"),
                (2, "u"),
                (2, "t"),
                (2, "s"),
                (1, "r"),
                (1, "q"),
                (1, "p"),
                (1, "o"),
                (1, "n"),
            ],
            result,
        );
    }

    #[test]
    fn test_analyse_single() {
        use crate::parse::sub_pattern::SubPatternDefinition;

        // Define input pattern
        let sub_patterns = vec![
            SubPatternDefinition::parse(parse_quote! { MyTupleStruct(foo, bar, baz) }).unwrap(),
        ];
        let pat_def = PatternDefinition {
            sub_patterns,
            span: Span::call_site(),
        };

        // Count occurences accross all subpatterns
        let result = IdentCounts::analyse(&pat_def).unwrap();

        // Assert correctness
        assert_positions(
            vec![(vec![1], "foo"), (vec![1], "bar"), (vec![1], "baz")],
            result,
        );
    }

    #[test]
    fn test_analyse_multiple_distinct() {
        use crate::parse::sub_pattern::SubPatternDefinition;

        // Define input patterns
        let sub_patterns = vec![
            SubPatternDefinition::parse(parse_quote! { MyTupleStruct(foo, bar, baz) }).unwrap(),
            SubPatternDefinition::parse(parse_quote! { MyTupleStructTwo(foo, qux) }).unwrap(),
            SubPatternDefinition::parse(parse_quote! { MyStruct { bar, quux } }).unwrap(),
        ];

        let pat_def = PatternDefinition {
            sub_patterns,
            span: Span::call_site(),
        };

        // Count occurences across all subpatterns
        let result = IdentCounts::analyse(&pat_def).unwrap();

        // Assert correctness
        assert_positions(
            vec![
                (vec![1, 1, 0], "foo"),
                (vec![1, 0, 1], "bar"),
                (vec![1, 0, 0], "baz"),
                (vec![0, 1, 0], "qux"),
                (vec![0, 0, 1], "quux"),
            ],
            result,
        );
    }

    #[test]
    fn test_analyse_multiple_overlapping() {
        use crate::parse::sub_pattern::SubPatternDefinition;

        // Define input patterns with overlapping idents
        let sub_patterns = vec![
            SubPatternDefinition::parse(parse_quote! { MyTupleStruct(foo, bar, bar) }).unwrap(),
            SubPatternDefinition::parse(parse_quote! { MyStruct { bar, baz } }).unwrap(),
        ];

        let pat_def = PatternDefinition {
            sub_patterns,
            span: Span::call_site(),
        };

        // Count occurences across all subpatterns
        let result = IdentCounts::analyse(&pat_def).unwrap();

        // Assert correctness
        assert_positions(
            vec![
                (vec![2, 1], "bar"),
                (vec![1, 0], "foo"),
                (vec![0, 1], "baz"),
            ],
            result,
        );
    }
}
