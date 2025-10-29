use syn::{Ident, Pat, PatIdent, PatPath, PatStruct, PatTupleStruct, Result};

pub trait SubPattern {
    fn get(&self) -> SubPatternDefinition;
    fn get_identifier(&self) -> &Ident;
}

impl SubPattern for SubPatternDefinition {
    fn get(&self) -> SubPatternDefinition {
        self.clone()
    }

    /// Get identifier that can be used to determine if two sub-patterns are of the same type.
    fn get_identifier(&self) -> &Ident {
        match self {
            SubPatternDefinition::Ident(ident) => &ident.ident,
            SubPatternDefinition::Path(p) => &p.path.segments.last().unwrap().ident,
            SubPatternDefinition::TupleStruct(p) => &p.path.segments.last().unwrap().ident,
            SubPatternDefinition::Struct(p) => &p.path.segments.last().unwrap().ident,
        }
    }
}

#[derive(Clone)]
/// A stand-alone part of a pattern, mapping to exactly one message.
pub enum SubPatternDefinition {
    Ident(PatIdent),
    Path(PatPath),
    TupleStruct(PatTupleStruct),
    Struct(PatStruct),
}

impl SubPatternDefinition {
    /// Parse a pattern into a SubPattern.
    /// Allowed constructs are Paths, TupleStructs, and Structs.
    pub fn parse(input: Pat) -> Result<Self> {
        match input {
            Pat::Ident(ident) => Ok(SubPatternDefinition::Ident(ident)),
            Pat::Path(path) => Ok(SubPatternDefinition::Path(path)),
            Pat::TupleStruct(tuple_struct) => Ok(SubPatternDefinition::TupleStruct(tuple_struct)),
            Pat::Struct(struct_pat) => Ok(SubPatternDefinition::Struct(struct_pat)),
            _ => Err(syn::Error::new_spanned(
                input,
                "Invalid construct in pattern",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Pat, parse_quote};

    #[test]
    fn test_parse_ident() {
        let pat: Pat = parse_quote!(SomeIdent);
        let result = SubPatternDefinition::parse(pat).unwrap();
        assert!(matches!(result, SubPatternDefinition::Ident(_)));
    }

    #[test]
    fn test_parse_path_simple() {
        let pat: Pat = parse_quote!(some::SomeVariant);
        let result = SubPatternDefinition::parse(pat).unwrap();
        assert!(matches!(result, SubPatternDefinition::Path(_)));
    }

    #[test]
    fn test_parse_tuple_struct() {
        let pat: Pat = parse_quote!(SomeTuple(x, y));
        let result = SubPatternDefinition::parse(pat).unwrap();
        assert!(matches!(result, SubPatternDefinition::TupleStruct(_)));
    }
    #[test]
    fn test_parse_struct() {
        let pat: Pat = parse_quote!(SomeStruct { x, y });
        let result = SubPatternDefinition::parse(pat).unwrap();
        assert!(matches!(result, SubPatternDefinition::Struct(_)));
    }

    #[test]
    fn test_parse_invalid() {
        let pat: Pat = parse_quote!((a, b));
        let result = SubPatternDefinition::parse(pat);
        assert!(result.is_err());
    }

    fn parse_to_identifier(pat: Pat) -> String {
        SubPatternDefinition::parse(pat)
            .unwrap()
            .get_identifier()
            .to_string()
    }

    #[test]
    fn test_identifier_equality_across_paths() {
        let pat1: Pat = parse_quote!(Variant);
        let pat2: Pat = parse_quote!(MyEnum::Variant);
        let pat3: Pat = parse_quote!(path::to::MyEnum::Variant);

        assert_eq!("Variant", parse_to_identifier(pat1));
        assert_eq!("Variant", parse_to_identifier(pat2));
        assert_eq!("Variant", parse_to_identifier(pat3));
    }

    #[test]
    fn test_identifier_equality_across_tuple_struct_paths() {
        let pat1: Pat = parse_quote!(TupleVariant(_, _));
        let pat2: Pat = parse_quote!(MyEnum::TupleVariant(_, _));
        let pat3: Pat = parse_quote!(path::to::MyEnum::TupleVariant(_, _));

        assert_eq!("TupleVariant", parse_to_identifier(pat1));
        assert_eq!("TupleVariant", parse_to_identifier(pat2));
        assert_eq!("TupleVariant", parse_to_identifier(pat3));
    }
}
