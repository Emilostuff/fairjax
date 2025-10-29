use crate::parse::pattern::{Pattern, PatternDefinition};
use crate::parse::strategy::Strategy;
use syn::{Arm, Expr, Result};

pub trait Case {
    fn index(&self) -> usize;
    fn strategy(&self) -> Strategy;
    fn pattern(&self) -> &dyn Pattern;
    fn guard(&self) -> Option<Expr>;
    fn body(&self) -> Expr;
}

impl Case for CaseDefinition {
    fn index(&self) -> usize {
        self.index
    }

    fn strategy(&self) -> Strategy {
        self.strategy.clone()
    }

    fn pattern(&self) -> &dyn Pattern {
        &self.pattern
    }

    fn guard(&self) -> Option<Expr> {
        self.guard.clone()
    }

    fn body(&self) -> Expr {
        self.body.clone()
    }
}

#[derive(Clone)]
pub struct CaseDefinition {
    pub index: usize,
    pub strategy: Strategy,
    pub pattern: PatternDefinition,
    pub guard: Option<Expr>,
    pub body: Expr,
}

impl CaseDefinition {
    /// Parse match Arm into case object
    pub fn parse(input: Arm, index: usize) -> Result<Self> {
        let Arm {
            attrs,
            pat,
            guard,
            body,
            ..
        } = input;

        // Check if case arm has a strategy attribute
        let strategy = Strategy::parse(attrs)?;

        // Parse match scrutinee into pattern object
        let pattern = PatternDefinition::parse(pat)?;

        // Retrieve guard expression if present
        let guard_option = match guard {
            Some((_, expr)) => Some(*expr),
            _ => None,
        };

        // Retrieve body and construct object
        Ok(CaseDefinition {
            index,
            strategy,
            pattern,
            guard: guard_option,
            body: *body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro_utils::assert_tokens;
    use quote::ToTokens;
    use syn::{Arm, parse_quote};

    #[test]
    fn test_parse_simple_case() {
        // Define input to test
        let input: Arm = parse_quote! {
            (A(x), B(y)) if x == y => println!("Success!")
        };

        // Define expected result
        let expected_strategy = Strategy::Auto;

        // Compute actual result
        let result = CaseDefinition::parse(input, 0).unwrap();

        // Check result
        assert_eq!(expected_strategy, result.strategy);
        assert_tokens!(result.guard.unwrap().to_token_stream(), { x == y });
        assert_tokens!(result.body.to_token_stream(), { println!("Success!") });
    }

    #[test]
    fn test_parse_case_without_guard() {
        // Define input to test
        let input: Arm = parse_quote! {
            (A(x), B(y)) => println!("No guard!")
        };

        // Define expected result
        let expected_strategy = Strategy::Auto;

        // Compute actual result
        let result = CaseDefinition::parse(input, 0).unwrap();

        // Check result
        assert_eq!(expected_strategy, result.strategy);
        assert!(result.guard.is_none());
        assert_tokens!(result.body.to_token_stream(), { println!("No guard!") });
    }

    #[test]
    fn test_parse_case_with_strategy_attribute() {
        // Define input to test
        let input: Arm = parse_quote! {
            #[BruteForce]
            (A(x), B(y)) => x + y
        };

        // Define expected result
        let expected_strategy = Strategy::BruteForce;

        // Compute actual result
        let result = CaseDefinition::parse(input, 0).unwrap();

        // Check result
        assert_eq!(expected_strategy, result.strategy);
        assert!(result.guard.is_none());
        assert_tokens!(result.body.to_token_stream(), { x + y });
    }

    #[test]
    fn test_parse_case_with_complex_guard() {
        // Define input to test
        let input: Arm = parse_quote! {
            (A(x), B(y)) if x > 0 && y < 10 => x * y
        };

        // Define expected result
        let expected_strategy = Strategy::Auto;

        // Compute actual result
        let result = CaseDefinition::parse(input, 0).unwrap();

        // Check result
        assert_eq!(expected_strategy, result.strategy);
        assert_tokens!(result.guard.unwrap().to_token_stream(), { x > 0 && y < 10 });
        assert_tokens!(result.body.to_token_stream(), { x * y });
    }

    #[test]
    fn test_parse_case_with_block_body() {
        // Define input to test
        let input: Arm = parse_quote! {
            (A(x), B(y)) => {
                let sum = x + y;
                println!("Sum: {}", sum);
                sum
            }
        };

        // Define expected result
        let expected_strategy = Strategy::Auto;

        // Compute actual result
        let result = CaseDefinition::parse(input, 0).unwrap();

        // Check result
        assert_eq!(expected_strategy, result.strategy);
        assert!(result.guard.is_none());
        assert_tokens!(result.body.to_token_stream(), {
            {
                let sum = x + y;
                println!("Sum: {}", sum);
                sum
            }
        });
    }
}
