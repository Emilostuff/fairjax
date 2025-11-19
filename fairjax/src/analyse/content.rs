use std::collections::BTreeMap;

use crate::parse::{pattern::PatternDefinition, sub_pattern::SubPatternDefinition};
use quote::ToTokens;
use syn::{Pat, PatSlice, PatTuple, PatTupleStruct};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructuralPath {
    path: Vec<String>,
    ambiguous: bool,
}

impl StructuralPath {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            ambiguous: false,
        }
    }

    pub fn new_with(&self, segments: &[String]) -> Self {
        let mut new = self.clone();
        new.path.extend_from_slice(segments);
        new
    }

    pub fn mark_ambiguous(mut self) -> Self {
        self.ambiguous = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubPatternContents {
    bindings: BTreeMap<String, Vec<StructuralPath>>,
    refutables: BTreeMap<StructuralPath, String>,
}

impl SubPatternContents {
    pub fn extract(pattern: &PatternDefinition) -> Vec<Self> {
        pattern
            .sub_patterns
            .iter()
            .map(Self::extract_from_sub_pattern)
            .collect()
    }

    pub fn extract_from_sub_pattern(sub_pattern: &SubPatternDefinition) -> Self {
        let mut contents = Self {
            bindings: BTreeMap::new(),
            refutables: BTreeMap::new(),
        };
        contents.extract_rec(&sub_pattern.to_syn_pattern(), StructuralPath::new());
        contents
    }

    fn extract_rec(&mut self, pat: &Pat, structural_path: StructuralPath) {
        use Pat::*;
        return match pat {
            Range(pat_range) => {
                self.refutables
                    .insert(structural_path, pat_range.to_token_stream().to_string());
            }
            Path(pat_path) => {
                self.refutables.insert(
                    structural_path,
                    pat_path.path.segments.last().unwrap().ident.to_string(),
                );
            }

            Lit(pat_lit) => {
                self.refutables
                    .insert(structural_path, pat_lit.lit.to_token_stream().to_string());
            }

            Ident(ident) => {
                let name = ident.ident.to_string();
                if name.chars().next().unwrap().is_uppercase() {
                    self.refutables.insert(structural_path, name);
                } else {
                    self.bindings
                        .entry(name)
                        .or_insert(Vec::new())
                        .push(structural_path);
                }
            }

            Paren(pat_paren) => {
                self.extract_rec(&pat_paren.pat, structural_path.new_with(&["()".into()]))
            }

            Struct(pat_struct) => pat_struct.fields.iter().for_each(|field| {
                self.extract_rec(
                    &field.pat,
                    structural_path.new_with(&[
                        pat_struct.path.segments.last().unwrap().ident.to_string(),
                        field.member.to_token_stream().to_string(),
                    ]),
                )
            }),

            Slice(PatSlice { elems, .. }) => elems.iter().enumerate().for_each(|(i, pat)| {
                self.extract_rec(
                    pat,
                    structural_path
                        .new_with(&["Slice".into(), i.to_string()])
                        .mark_ambiguous(),
                )
            }),

            Tuple(PatTuple { elems, .. }) => elems.iter().enumerate().for_each(|(i, pat)| {
                self.extract_rec(
                    pat,
                    structural_path.new_with(&["Tuple".into(), i.to_string()]),
                )
            }),

            TupleStruct(PatTupleStruct { elems, path, .. }) => {
                elems.iter().enumerate().for_each(|(i, pat)| {
                    self.extract_rec(
                        pat,
                        structural_path.new_with(&[
                            path.segments.last().unwrap().ident.to_string(),
                            i.to_string(),
                        ]),
                    )
                })
            }

            Or(pat_or) => pat_or.cases.iter().enumerate().for_each(|(i, pat)| {
                self.extract_rec(pat, structural_path.new_with(&["Or".into(), i.to_string()]))
            }),

            // Ignore all other Pat variants (They are either not applicable or invalid and reported elsewhere)
            &_ => (),
        };
    }

    pub fn same_refutables(a: &Self, b: &Self) -> bool {
        a.refutables == b.refutables
    }

    pub fn same_placements(binding: &String, a: &Self, b: &Self) -> bool {
        match (a.bindings.get(binding), b.bindings.get(binding)) {
            (Some(path_a), Some(path_b)) => path_a == path_b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::Pat;
    use syn::parse_quote;

    fn sp(path: Vec<&str>, ambiguous: bool) -> StructuralPath {
        StructuralPath {
            path: path.iter().map(|&s| s.to_string()).collect(),
            ambiguous,
        }
    }

    #[test]
    fn test_parse_single() {
        let input: Pat = parse_quote! {
            A {
                a: SomeTuple(x, _, "foo"),
                b: 1 | 2 | 3,
                c: 42..69,
                d: EnumVariant,
                e
            }
        };
        let pattern = PatternDefinition::parse(input).unwrap();
        let result = SubPatternContents::extract(&pattern);

        let expected = SubPatternContents {
            bindings: BTreeMap::from([
                (
                    "x".to_string(),
                    vec![sp(vec!["A", "a", "SomeTuple", "0"], false)],
                ),
                ("e".to_string(), vec![sp(vec!["A", "e"], false)]),
            ]),
            refutables: BTreeMap::from([
                (
                    sp(vec!["A", "a", "SomeTuple", "2"], false),
                    "\"foo\"".to_string(),
                ),
                (sp(vec!["A", "b", "Or", "0"], false), "1".to_string()),
                (sp(vec!["A", "b", "Or", "1"], false), "2".to_string()),
                (sp(vec!["A", "b", "Or", "2"], false), "3".to_string()),
                (sp(vec!["A", "c"], false), "42 .. 69".to_string()),
                (sp(vec!["A", "d"], false), "EnumVariant".to_string()),
            ]),
        };

        assert_eq!(result[0], expected);
    }

    #[test]
    fn test_parse_single_duplicate_bindings() {
        let input: Pat = parse_quote! {
            A {
                a: SomeTuple(x, _, x),
                b: x | x,
                c: Some(x),
                d: [x, .., x],
                x
            }
        };
        let pattern = PatternDefinition::parse(input).unwrap();
        let result = SubPatternContents::extract(&pattern);

        let expected = SubPatternContents {
            bindings: BTreeMap::from([(
                "x".to_string(),
                vec![
                    sp(vec!["A", "a", "SomeTuple", "0"], false),
                    sp(vec!["A", "a", "SomeTuple", "2"], false),
                    sp(vec!["A", "b", "Or", "0"], false),
                    sp(vec!["A", "b", "Or", "1"], false),
                    sp(vec!["A", "c", "Some", "0"], false),
                    sp(vec!["A", "d", "Slice", "0"], true),
                    sp(vec!["A", "d", "Slice", "2"], true),
                    sp(vec!["A", "x"], false),
                ],
            )]),
            refutables: BTreeMap::new(),
        };

        assert_eq!(result[0], expected);
    }

    #[test]
    fn test_parse_single_duplicate_refutables() {
        let input: Pat = parse_quote! {
            A {
                a: SomeTuple(1, _, 1),
                b: 1 | 1,
                c: Some(1),
                d: [1, .., 1],
                x: 1
            }
        };
        let pattern = PatternDefinition::parse(input).unwrap();
        let result = SubPatternContents::extract(&pattern);

        let expected = SubPatternContents {
            bindings: BTreeMap::new(),
            refutables: BTreeMap::from([
                (sp(vec!["A", "a", "SomeTuple", "0"], false), "1".into()),
                (sp(vec!["A", "a", "SomeTuple", "2"], false), "1".into()),
                (sp(vec!["A", "b", "Or", "0"], false), "1".into()),
                (sp(vec!["A", "b", "Or", "1"], false), "1".into()),
                (sp(vec!["A", "c", "Some", "0"], false), "1".into()),
                (sp(vec!["A", "d", "Slice", "0"], true), "1".into()),
                (sp(vec!["A", "d", "Slice", "2"], true), "1".into()),
                (sp(vec!["A", "x"], false), "1".into()),
            ]),
        };

        assert_eq!(result[0], expected);
    }
}
