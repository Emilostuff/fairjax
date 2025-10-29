use quote::ToTokens;
use syn::{BinOp, Error, Expr, Path, Result};

#[derive(Clone)]
pub struct Context {
    pub incoming_message: Expr,
    pub mailbox: Expr,
    pub message_type: Path,
}

impl Context {
    pub fn parse(input: Expr) -> Result<Self> {
        // Check that top-level expression is a binary expression
        let expr_binary = match input {
            Expr::Binary(expr_binary) => expr_binary,
            _ => {
                return Err(Error::new_spanned(
                    input,
                    "Expected syntax: <MESSAGE> >> [<MAILBOX>, <MESSAGE_TYPE>]",
                ));
            }
        };

        // Check that operator in expression is '>>'
        if !matches!(expr_binary.op, BinOp::Shr(_)) {
            return Err(Error::new_spanned(
                expr_binary.op.clone(),
                format!(
                    "Expected '>>' operator, found: {}",
                    expr_binary.op.to_token_stream().to_string()
                ),
            ));
        }

        // Take incoming message expr
        let incoming_message = *expr_binary.left;

        // Unpack array
        let (mailbox, message_type_expr) = match *expr_binary.right {
            Expr::Array(array) if array.elems.len() == 2 => {
                let mut elems = array.elems.into_iter();
                (elems.next().unwrap(), elems.next().unwrap())
            }
            other => {
                return Err(Error::new_spanned(
                    other,
                    "Expected '[<MAILBOX>, <MESSAGE_TYPE>]' here",
                ));
            }
        };

        // Verify that message type is a path
        let message_type = match message_type_expr {
            Expr::Path(expr_path) => expr_path.path,
            other => {
                return Err(Error::new_spanned(
                    other,
                    "Expected MESSAGE_TYPE to be a path",
                ));
            }
        };

        Ok(Context {
            incoming_message,
            mailbox,
            message_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro_utils::assert_tokens;
    use syn::{Expr, parse_quote};

    #[test]
    fn test_parse_simple() {
        // Define input to test
        let input: Expr = parse_quote!(msg >> [mailbox, MyMessage]);

        // Compute actual result
        let result = Context::parse(input).unwrap();

        // Check that they are equal
        assert_tokens!(result.incoming_message.to_token_stream(), { msg });
        assert_tokens!(result.mailbox.to_token_stream(), { mailbox });
        assert_tokens!(result.message_type.to_token_stream(), { MyMessage });
    }

    #[test]
    fn test_parse_longer_path() {
        // Define input to test
        let input: Expr = parse_quote!(msg >> [mailbox, some::long::path::to::MyMessage]);

        // Define expected result
        let result = Context::parse(input).unwrap();

        assert_tokens!(result.incoming_message.to_token_stream(), { msg });
        assert_tokens!(result.mailbox.to_token_stream(), { mailbox });
        assert_tokens!(result.message_type.to_token_stream(), {
            some::long::path::to::MyMessage
        });
    }

    #[test]
    fn test_parse_message_clone() {
        let input: Expr = parse_quote!(msg.clone() >> [mailbox, MyMessage]);
        let result = Context::parse(input).unwrap();

        assert_tokens!(result.incoming_message.to_token_stream(), { msg.clone() });
        assert_tokens!(result.mailbox.to_token_stream(), { mailbox });
        assert_tokens!(result.message_type.to_token_stream(), { MyMessage });
    }

    #[test]
    fn test_parse_mailbox_on_self() {
        // Define input to test
        let input: Expr = parse_quote!(msg >> [self.mailbox, MyMessage]);

        // Compute actual result
        let result = Context::parse(input).unwrap();

        // Check that they are equal
        assert_tokens!(result.incoming_message.to_token_stream(), { msg });
        assert_tokens!(result.mailbox.to_token_stream(), { self.mailbox });
        assert_tokens!(result.message_type.to_token_stream(), { MyMessage });
    }

    #[test]
    fn test_parse_mailbox_ref() {
        // Define input to test
        let input: Expr = parse_quote!(msg >> [mailbox.as_ref(), MyMessage]);

        // Compute actual result
        let result = Context::parse(input).unwrap();

        // Check that they are equal
        assert_tokens!(result.incoming_message.to_token_stream(), { msg });
        assert_tokens!(result.mailbox.to_token_stream(), { mailbox.as_ref() });
        assert_tokens!(result.message_type.to_token_stream(), { MyMessage });
    }
}
