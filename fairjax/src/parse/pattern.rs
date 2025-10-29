use crate::parse::sub_pattern::{SubPattern, SubPatternDefinition};
use syn::{Pat, Result};

pub trait Pattern {
    fn sub_patterns(&self) -> Vec<&dyn SubPattern>;
    fn len(&self) -> usize;
}

impl Pattern for PatternDefinition {
    fn sub_patterns(&self) -> Vec<&dyn SubPattern> {
        self.0.iter().map(|sp| sp as &dyn SubPattern).collect()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone)]
// The pattern of a match arm expression
pub struct PatternDefinition(pub Vec<SubPatternDefinition>);

impl PatternDefinition {
    /// Parse a pattern from a Pat obejct into a Pattern
    /// Tries to parse the pattern as either a Tuple of SubPatterns or a single SubPattern.
    pub fn parse(input: Pat) -> Result<Self> {
        match input.clone() {
            Pat::Tuple(tuple) => {
                let sub_patterns = tuple
                    .elems
                    .into_iter()
                    .map(|p| SubPatternDefinition::parse(p))
                    .collect::<Result<Vec<SubPatternDefinition>>>()?;
                if sub_patterns.len() > 0 {
                    Ok(PatternDefinition(sub_patterns))
                } else {
                    Err(syn::Error::new_spanned(
                        input,
                        "Empty pattern is not allowed",
                    ))
                }
            }
            singleton => Ok(PatternDefinition(vec![SubPatternDefinition::parse(
                singleton,
            )?])),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Pat, parse_quote};

    #[test]
    fn test_parse_single() {
        let input: Pat = parse_quote!(MyStruct);
        assert!(PatternDefinition::parse(input).is_ok());
    }

    #[test]
    fn test_parse_tuple() {
        let input: Pat = parse_quote!((MyStruct, MyStruct, MyStruct));
        assert!(PatternDefinition::parse(input).is_ok());
    }

    #[test]
    fn test_parse_invalid_tuple_fails() {
        let input: Pat = parse_quote!((MyStruct, 42, MyStruct));
        assert!(PatternDefinition::parse(input).is_err());
    }

    #[test]
    fn test_parse_empty_tuple_fails() {
        let input: Pat = parse_quote!(());
        assert!(PatternDefinition::parse(input).is_err());
    }

    #[test]
    fn test_parse_wildcard_fails() {
        let input: Pat = parse_quote!(_);
        assert!(PatternDefinition::parse(input).is_err());
    }

    #[test]
    fn test_parse_slice_fails() {
        let input: Pat = parse_quote!([a, b, c]);
        assert!(PatternDefinition::parse(input).is_err());
    }
}
