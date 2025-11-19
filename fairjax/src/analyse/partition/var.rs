use super::counts::{IdentCounts, SubPatternOccurrences};
use syn::{Error, Result};

pub struct Accumulator {
    partition_vars: Vec<String>,
    errors: Option<Error>,
}

impl Accumulator {
    fn new() -> Self {
        Accumulator {
            partition_vars: vec![],
            errors: None,
        }
    }

    fn with_partition_var(mut self, name: String) -> Self {
        self.partition_vars.push(name);
        self
    }

    fn with_errors(mut self, message: String, occurrences: Vec<SubPatternOccurrences>) -> Self {
        let combined_error = occurrences
            .into_iter()
            .flat_map(|occ| occ.0)
            .map(|span| Error::new(span, message.clone()))
            .reduce(|mut acc, err| {
                acc.combine(err);
                acc
            });

        if let Some(new_errors) = combined_error {
            self.errors = self
                .errors
                .take()
                .map_or(Some(new_errors.clone()), |mut errors| {
                    errors.combine(new_errors);
                    Some(errors)
                });
        };
        self
    }
}

pub struct PartitionVars;

impl PartitionVars {
    pub fn identify(idents: IdentCounts, pattern_size: usize) -> Result<Vec<String>> {
        let result = idents
            .into_iter()
            .fold(Accumulator::new(), |acc, (name, occurrences)| {
                Self::check(acc, pattern_size, name, occurrences)
            });

        match result.errors {
            Some(err) => Err(err),
            None => Ok(result.partition_vars),
        }
    }

    /// Determine if a valid ident is a variable
    fn is_var(input: &String) -> bool {
        input
            .chars()
            .all(|c| c.is_lowercase() || !c.is_alphabetic())
    }

    /// Count the number of occurrences of a variable in a pattern
    fn count(occurrences: &Vec<SubPatternOccurrences>) -> usize {
        occurrences.iter().map(|occ| occ.len()).sum()
    }

    /// Determine if a variable is present in all sub-patterns
    fn in_all(occurrences: &Vec<SubPatternOccurrences>, pattern_size: usize) -> bool {
        occurrences.iter().all(|occ| occ.len() > 0) && occurrences.len() == pattern_size
    }

    /// Locate duplicates of a variable in one or more sub-patterns
    fn duplicates(occurrences: &Vec<SubPatternOccurrences>) -> Option<Vec<SubPatternOccurrences>> {
        let mut sub_pattern_with_dups = Vec::new();
        for occ in occurrences {
            if occ.len() > 1 {
                sub_pattern_with_dups.push(occ.clone());
            }
        }
        if sub_pattern_with_dups.is_empty() {
            None
        } else {
            Some(sub_pattern_with_dups)
        }
    }

    pub fn check(
        acc: Accumulator,
        pattern_size: usize,
        name: String,
        occ: Vec<SubPatternOccurrences>,
    ) -> Accumulator {
        // Ignore if not present more than once or not a variable (could be an enum variant ident)
        if Self::count(&occ) < 2 || !Self::is_var(&name) {
            return acc;
        }

        // Throw helpful error if variable is not present in all sub patterns
        if !Self::in_all(&occ, pattern_size) {
            return acc.with_errors(
                format!(
                    "'{}' must occur exactly once in every \
                        message pattern to be a partition variable",
                    name
                ),
                occ,
            );
        }

        // Check for duplicates within a single sub pattern
        if let Some(dups) = Self::duplicates(&occ) {
            return acc.with_errors(
                format!(
                    "Multiple occurences of partition variable '{}' \
                            within a single message pattern.",
                    name
                ),
                dups,
            );
        }

        // Variable is a valid partition variable
        acc.with_partition_var(name)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use proc_macro2::Span;

    use super::*;

    #[test]
    fn test_is_var_correct() {
        assert!(PartitionVars::is_var(&"x".into()));
        assert!(PartitionVars::is_var(&"var_name".into()));
        assert!(PartitionVars::is_var(&"name123".into()));
        assert!(PartitionVars::is_var(&"_unused_var_name".into()));
    }

    #[test]
    fn test_is_var_incorrect() {
        assert!(!PartitionVars::is_var(&"X".into()));
        assert!(!PartitionVars::is_var(&"MyEnumVariant".into()));
        assert!(!PartitionVars::is_var(&"Test1".into()));
        assert!(!PartitionVars::is_var(&"weirdCamelCaseIdent".into()));
        assert!(!PartitionVars::is_var(&"SOME_CONSTANT123".into()));
    }

    #[test]
    fn test_count_basic() {
        // 3 sub-patterns, with 1, 2, and 0 occurrences respectively
        let occurrences = get_dummy_occurrences(vec![1, 2, 0]);
        assert_eq!(PartitionVars::count(&occurrences), 3);
    }

    #[test]
    fn test_count_empty() {
        let occurrences = get_dummy_occurrences(vec![0, 0, 0]);
        assert_eq!(PartitionVars::count(&occurrences), 0);
    }

    #[test]
    fn test_in_all_true() {
        let pattern_size = 3;
        let occurrences = get_dummy_occurrences(vec![1, 1, 1]);
        assert!(PartitionVars::in_all(&occurrences, pattern_size));
    }

    #[test]
    fn test_in_all_true_with_dups() {
        let pattern_size = 3;
        let occurrences = get_dummy_occurrences(vec![1, 2, 3]);
        assert!(PartitionVars::in_all(&occurrences, pattern_size));
    }

    #[test]
    fn test_in_all_false_missing() {
        let pattern_size = 3;
        let occurrences = get_dummy_occurrences(vec![1, 0, 1]);
        assert!(!PartitionVars::in_all(&occurrences, pattern_size));
    }

    #[test]
    fn test_duplicates_none() {
        let occurrences = get_dummy_occurrences(vec![1, 1, 1]);
        assert!(PartitionVars::duplicates(&occurrences).is_none());
    }

    #[test]
    fn test_duplicates_some() {
        let occurrences = get_dummy_occurrences(vec![1, 2, 1]);
        let dups = PartitionVars::duplicates(&occurrences);
        let dups_vec = dups.unwrap();
        assert_eq!(dups_vec.len(), 1);
        assert_eq!(dups_vec[0].len(), 2);
    }

    #[test]
    fn test_duplicates_multiple() {
        let occurrences = get_dummy_occurrences(vec![2, 2, 1]);
        let dups = PartitionVars::duplicates(&occurrences);
        let dups_vec = dups.unwrap();
        assert_eq!(dups_vec.len(), 2);
        assert_eq!(dups_vec[0].len(), 2);
        assert_eq!(dups_vec[1].len(), 2);
    }

    fn get_dummy_occurrences(occurrences: Vec<usize>) -> Vec<SubPatternOccurrences> {
        occurrences
            .iter()
            .map(|n| SubPatternOccurrences(vec![Span::call_site(); *n]))
            .collect()
    }

    #[test]
    fn test_check_valid_candidate() {
        let pattern_size = 6;
        let name = "x".into();
        let occurrences = get_dummy_occurrences(vec![1, 1, 1, 1, 1, 1]);

        let result = PartitionVars::check(Accumulator::new(), pattern_size, name, occurrences);
        assert_eq!(vec!["x"], result.partition_vars);
    }

    #[test]
    fn test_check_ignore_singleton() {
        let pattern_size = 4;
        let name = "x".into();
        let occurrences = get_dummy_occurrences(vec![0, 1, 0, 0]);

        let result = PartitionVars::check(Accumulator::new(), pattern_size, name, occurrences);
        assert!(result.partition_vars.is_empty());
    }

    #[test]
    fn test_check_ignore_enum_variant() {
        let pattern_size = 4;
        let name = "EnumVariant".into();
        let occurrences = get_dummy_occurrences(vec![1, 2, 3, 4]);

        let result = PartitionVars::check(Accumulator::new(), pattern_size, name, occurrences);
        assert!(result.partition_vars.is_empty());
    }

    #[test]
    fn test_check_fail_on_incomplete_repetition() {
        let pattern_size = 4;
        let name = "x".into();
        let occurrences = get_dummy_occurrences(vec![1, 1, 0, 1]);

        let result = PartitionVars::check(Accumulator::new(), pattern_size, name, occurrences);
        assert!(result.partition_vars.is_empty());
        assert!(result.errors.is_some());
    }

    #[test]
    fn test_check_duplicates() {
        let pattern_size = 4;
        let name = "x".into();
        let occurrences = get_dummy_occurrences(vec![2, 0, 0, 0]);

        let result = PartitionVars::check(Accumulator::new(), pattern_size, name, occurrences);
        assert!(result.partition_vars.is_empty());
        assert!(result.errors.is_some());
    }

    #[test]
    fn test_check_in_all_but_wit_duplicates() {
        let pattern_size = 4;
        let name = "x".into();
        let occurrences = get_dummy_occurrences(vec![1, 1, 2, 1]);

        let result = PartitionVars::check(Accumulator::new(), pattern_size, name, occurrences);
        assert!(result.partition_vars.is_empty());
        assert!(result.errors.is_some());
    }

    #[test]
    fn test_identify_single() {
        let pattern_size = 4;
        let ident_counts = IdentCounts {
            pattern_size,
            counts: BTreeMap::from_iter(
                vec![
                    ("x".to_string(), get_dummy_occurrences(vec![0, 1, 0, 0])),
                    ("y".to_string(), get_dummy_occurrences(vec![1, 1, 1, 1])),
                    ("z".to_string(), get_dummy_occurrences(vec![0, 0, 0, 1])),
                ]
                .into_iter(),
            ),
        };
        let result = PartitionVars::identify(ident_counts, pattern_size).unwrap();
        assert_eq!(vec!["y"], result);
    }

    #[test]
    fn test_identify_multiple_with_noise() {
        let pattern_size = 4;
        let ident_counts = IdentCounts {
            pattern_size,
            counts: BTreeMap::from_iter(
                vec![
                    ("x".to_string(), get_dummy_occurrences(vec![0, 0, 1, 0])),
                    ("y".to_string(), get_dummy_occurrences(vec![1, 1, 1, 1])),
                    ("z".to_string(), get_dummy_occurrences(vec![1, 1, 1, 1])),
                    ("A".to_string(), get_dummy_occurrences(vec![2, 0, 5, 1])), // not a variable name
                ]
                .into_iter(),
            ),
        };
        let result = PartitionVars::identify(ident_counts, pattern_size).unwrap();
        assert_eq!(vec!["y", "z"], result);
    }

    #[test]
    fn test_identify_none() {
        let pattern_size = 4;
        let ident_counts = IdentCounts {
            pattern_size,
            counts: BTreeMap::from_iter(
                vec![
                    ("x".to_string(), get_dummy_occurrences(vec![0, 0, 1, 0])),
                    ("y".to_string(), get_dummy_occurrences(vec![1, 0, 0, 0])),
                    ("z".to_string(), get_dummy_occurrences(vec![0, 0, 0, 1])),
                    ("A".to_string(), get_dummy_occurrences(vec![2, 0, 5, 1])), // not a variable name
                ]
                .into_iter(),
            ),
        };
        let result = PartitionVars::identify(ident_counts, pattern_size).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_identify_failin() {
        let pattern_size = 4;
        let ident_counts = IdentCounts {
            pattern_size,
            counts: BTreeMap::from_iter(
                vec![
                    ("x".to_string(), get_dummy_occurrences(vec![0, 0, 1, 0])),
                    ("y".to_string(), get_dummy_occurrences(vec![1, 0, 0, 0])),
                    ("z".to_string(), get_dummy_occurrences(vec![0, 0, 1, 1])),
                ]
                .into_iter(),
            ),
        };
        let result = PartitionVars::identify(ident_counts, pattern_size);
        assert!(result.is_err());
    }
}
