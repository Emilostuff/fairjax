use syn::{Attribute, Meta, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputStrategy {
    Auto,
    StatefulTree,
    BruteForce,
    Partitions,
}

impl InputStrategy {
    /// Parse a strategy from a vector of attributes.
    /// Checks that exactly 0 or 1 attribute is provided. Defaults to Auto when no attribute is provided.
    pub fn parse(input: Vec<Attribute>) -> Result<Self> {
        match input.len() {
            0 => Ok(InputStrategy::Auto),
            1 => {
                if let Meta::Path(strategy_ident) = &input[0].meta {
                    match strategy_ident.get_ident() {
                        Some(ident) if ident == "Auto" => Ok(InputStrategy::Auto),
                        Some(ident) if ident == "StatefulTree" => Ok(InputStrategy::StatefulTree),
                        Some(ident) if ident == "BruteForce" => Ok(InputStrategy::BruteForce),
                        Some(ident) if ident == "Partitions" => Ok(InputStrategy::Partitions),
                        _ => Err(syn::Error::new_spanned(
                            strategy_ident,
                            "Invalid strategy name",
                        )),
                    }
                } else {
                    Err(syn::Error::new_spanned(
                        &input[0],
                        "Invalid stratety declaration",
                    ))
                }
            }
            _ => Err(syn::Error::new_spanned(
                &input[1],
                "Only one attribute allowed",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::format_ident;
    use syn::{Attribute, parse_quote};

    fn attr_from_ident(ident: &str) -> Attribute {
        let ident = format_ident!("{}", ident);
        // Create an attribute like #[Auto], #[StatefulTree], #[BruteForce]
        parse_quote!(#[#ident])
    }

    #[test]
    fn test_parse_empty_vec_returns_auto() {
        let attrs: Vec<Attribute> = vec![];
        let result = InputStrategy::parse(attrs).unwrap();
        assert_eq!(result, InputStrategy::Auto);
    }

    #[test]
    fn test_parse_auto() {
        let attrs = vec![attr_from_ident("Auto")];
        let result = InputStrategy::parse(attrs).unwrap();
        assert_eq!(result, InputStrategy::Auto);
    }

    #[test]
    fn test_parse_stateful_tree() {
        let attrs = vec![attr_from_ident("StatefulTree")];
        let result = InputStrategy::parse(attrs).unwrap();
        assert_eq!(result, InputStrategy::StatefulTree);
    }

    #[test]
    fn test_parse_brute_force() {
        let attrs = vec![attr_from_ident("BruteForce")];
        let result = InputStrategy::parse(attrs).unwrap();
        assert_eq!(result, InputStrategy::BruteForce);
    }

    #[test]
    fn test_parse_partitions() {
        let attrs = vec![attr_from_ident("Partitions")];
        let result = InputStrategy::parse(attrs).unwrap();
        assert_eq!(result, InputStrategy::Partitions);
    }

    #[test]
    fn test_parse_invalid_strategy_name() {
        let attrs = vec![attr_from_ident("UnknownStrategy")];
        let result = InputStrategy::parse(attrs);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Invalid strategy name"));
    }

    #[test]
    fn test_parse_invalid_meta_type() {
        // Create an attribute with a meta that is not a path (e.g., a name-value pair)
        let attr: Attribute = parse_quote!(#[strategy = "Auto"]);
        let attrs = vec![attr];
        let result = InputStrategy::parse(attrs);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Invalid stratety declaration"));
    }

    #[test]
    fn test_parse_multiple_attributes() {
        let attrs = vec![attr_from_ident("Auto"), attr_from_ident("StatefulTree")];
        let result = InputStrategy::parse(attrs);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Only one attribute allowed"));
    }
}
