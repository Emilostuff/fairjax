use crate::traits::Pattern;
use std::collections::BTreeMap;

#[derive(Clone)]
/// Statistics for a sub-pattern: how many times it appears and where
pub struct Group {
    occurrences: Vec<usize>,
}

impl Group {
    pub fn new(index: usize) -> Self {
        Self {
            occurrences: vec![index],
        }
    }

    pub fn size(&self) -> usize {
        self.occurrences.len()
    }

    pub fn occurrences(&self) -> &Vec<usize> {
        &self.occurrences
    }

    pub fn first(&self) -> usize {
        *self
            .occurrences
            .first()
            .expect("Vec has at least one element")
    }

    pub fn push(&mut self, index: usize) {
        self.occurrences.push(index);
    }
}

/// Analyzes a pattern to collect statistics about all its sub-patterns
pub struct SubPatternGroups(pub Vec<Group>);

impl SubPatternGroups {
    pub fn new(pattern: &dyn Pattern) -> Self {
        // Perform analysis on Sub Patterns, tracking their positions and occurences in the pattern
        let mut sp_stats: BTreeMap<String, Group> = BTreeMap::new();

        for (position, &sub_pattern) in pattern.sub_patterns().iter().enumerate() {
            let identifier = sub_pattern.get_identifier().to_string();

            sp_stats
                .entry(identifier.clone())
                .and_modify(|stats| {
                    stats.push(position);
                })
                .or_insert(Group::new(position));
        }

        Self(sp_stats.into_values().collect())
    }

    /// Pattern does not have any repeated message variants
    pub fn is_distinct(&self) -> bool {
        self.0.iter().all(|group| group.size() == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::sub_pattern::SubPatternDefinition;
    use crate::traits::SubPattern;
    use proc_macro2::Span;
    use syn::Ident;

    fn ident(input: &str) -> Ident {
        Ident::new(input, proc_macro2::Span::call_site())
    }

    struct MockSubPattern(syn::Ident);

    impl SubPattern for MockSubPattern {
        fn get(&self) -> SubPatternDefinition {
            unimplemented!() //
        }

        fn get_identifier(&self) -> &Ident {
            &self.0
        }
    }

    struct MockPattern {
        sub_patterns: Vec<MockSubPattern>,
    }

    impl MockPattern {
        fn new(sub_pattern_idents: &[&'static str]) -> Self {
            Self {
                sub_patterns: sub_pattern_idents
                    .into_iter()
                    .map(|name| MockSubPattern(ident(name)))
                    .collect(),
            }
        }
    }

    impl Pattern for MockPattern {
        fn sub_patterns(&self) -> Vec<&dyn SubPattern> {
            self.sub_patterns
                .iter()
                .map(|sp| sp as &dyn SubPattern)
                .collect()
        }

        fn len(&self) -> usize {
            unimplemented!()
        }

        fn span(&self) -> Span {
            Span::call_site()
        }
    }

    // Tests
    #[test]
    fn test_single() {
        let pattern = MockPattern::new(&["X"]);
        let result = SubPatternGroups::new(&pattern);

        // Assert that only a single stat is generated
        assert_eq!(1, result.0.len());

        // Check the stats
        let s0 = result.0[0].clone();
        assert_eq!(0, s0.first());
        assert_eq!(1, s0.size());
        assert_eq!(vec![0], s0.occurrences);
    }

    #[test]
    fn test_multiple_simple() {
        let pattern = MockPattern::new(&["A", "A", "B"]);
        let result = SubPatternGroups::new(&pattern);

        // Assert that exactly two stats are generated
        assert_eq!(2, result.0.len());

        // Check the stats
        let s0 = result.0[0].clone();
        assert_eq!(0, s0.first());
        assert_eq!(2, s0.size());
        assert_eq!(vec![0, 1], s0.occurrences);

        let s1 = result.0[1].clone();
        assert_eq!(2, s1.first());
        assert_eq!(1, s1.size());
        assert_eq!(vec![2], s1.occurrences);
    }

    #[test]
    fn test_multiple_complex_in_order() {
        let pattern =
            MockPattern::new(&["A", "B", "A", "C", "B", "D", "A", "C", "E", "B", "D", "A"]);
        let result = SubPatternGroups::new(&pattern);

        // Assert that exactly 5 stats are generated (A, B, C, D, E)
        assert_eq!(5, result.0.len());

        // Check stats
        let s0 = &result.0[0];
        assert_eq!(0, s0.first());
        assert_eq!(4, s0.size());
        assert_eq!(vec![0, 2, 6, 11], s0.occurrences);

        let s1 = &result.0[1];
        assert_eq!(1, s1.first());
        assert_eq!(3, s1.size());
        assert_eq!(vec![1, 4, 9], s1.occurrences);

        let s2 = &result.0[2];
        assert_eq!(3, s2.first());
        assert_eq!(2, s2.size());
        assert_eq!(vec![3, 7], s2.occurrences);

        let s3 = &result.0[3];
        assert_eq!(5, s3.first());
        assert_eq!(2, s3.size());
        assert_eq!(vec![5, 10], s3.occurrences);

        let s4 = &result.0[4];
        assert_eq!(8, s4.first());
        assert_eq!(1, s4.size());
        assert_eq!(vec![8], s4.occurrences);
    }
}
