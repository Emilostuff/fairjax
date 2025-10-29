use crate::compile::pattern::full::PatternCodeGen;
use crate::compile::pattern::sub::SubPatternCompiler;
use crate::parse::{case::Case, context::Context, pattern::Pattern};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote_spanned};
use syn::{Ident, spanned::Spanned};

pub trait GuardCodeGen {
    fn generate<'a, P: PatternCodeGen>(
        case: &dyn Case,
        context: &Context,
        fn_name: &'a str,
    ) -> TokenStream;
}

pub struct GuardCompiler;

impl GuardCodeGen for GuardCompiler {
    fn generate<'a, P: PatternCodeGen>(
        case: &dyn Case,
        context: &Context,
        fn_name: &'a str,
    ) -> TokenStream {
        let span = match case.guard() {
            Some(guard) => guard.span(),
            None => case.span(),
        };

        // Construct guard function identifier
        let fn_ident = Ident::new(fn_name, span);

        // Define standardized function parameter names
        let messages_param_ident = Ident::new("messages", span);
        let mapping_param_ident = Ident::new("mapping", span);

        // Retrieve values for code block
        let message_type = &context.message_type;
        let pattern_len = case.pattern().len();

        // Generate code snippets
        let unpacking = GuardCompiler::unpacking_code(
            span,
            case.pattern(),
            &messages_param_ident,
            &mapping_param_ident,
        );
        let guard_expr = GuardCompiler::guard_expression_code(span, case.guard());
        let pattern = P::generate::<SubPatternCompiler>(case.pattern());

        quote_spanned! {
            span =>
            fn #fn_ident(
                #messages_param_ident: &[&#message_type; #pattern_len],
                #mapping_param_ident: &fairjax_core::Mapping<#pattern_len>,
            ) -> bool {
                match #unpacking {
                    #pattern => #guard_expr,
                    _ => false,
                }
            }
        }
    }
}

impl GuardCompiler {
    /// Generate unpacking code that maps messages from their stored position given a mapping
    fn unpacking_code(
        span: Span,
        pattern: &dyn Pattern,
        message_param_ident: &Ident,
        mapping_param_ident: &Ident,
    ) -> TokenStream {
        // If there is only one match scrutinee, omit parenthesis
        if pattern.len() == 1 {
            return quote_spanned!( span => #message_param_ident[#mapping_param_ident.get(0usize)] );
        }

        // Otherwise wrap the match scrutinee expression in parenthesis
        let indices = 0..pattern.len();
        quote_spanned!( span => (#(#message_param_ident[#mapping_param_ident.get(#indices)]),*,) )
    }

    /// Generates the guard evaluation expression code.
    /// If the guard is `None` the evaluation expression is always `true`.
    fn guard_expression_code(span: Span, guard_expr: Option<syn::Expr>) -> TokenStream {
        match guard_expr {
            Some(expr) => expr.to_token_stream(),
            None => quote_spanned!(span => true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::pattern::sub::SubPatternCodeGen;
    use crate::parse::{case::Case, pattern::Pattern, strategy::Strategy, sub_pattern::SubPattern};
    use proc_macro_utils::assert_tokens;
    use proc_macro2::TokenStream;
    use quote::{ToTokens, format_ident};
    use syn::{Expr, parse_quote};

    // Mock Pattern with N sub-patterns
    struct MockPattern<const N: usize>;

    impl<const N: usize> Pattern for MockPattern<N> {
        fn sub_patterns(&self) -> Vec<&dyn SubPattern> {
            unimplemented!()
        }

        fn len(&self) -> usize {
            N
        }

        fn span(&self) -> Span {
            Span::call_site()
        }
    }

    // Mock Case with N sub-patterns
    struct MockCase<const N: usize> {
        pattern: MockPattern<N>,
        guard: Option<Expr>,
    }

    impl<const N: usize> MockCase<N> {
        fn new(guard: Option<Expr>) -> Self {
            MockCase {
                pattern: MockPattern::<N> {},
                guard,
            }
        }
    }

    impl<const N: usize> Case for MockCase<N> {
        fn index(&self) -> usize {
            unimplemented!()
        }
        fn strategy(&self) -> Strategy {
            unimplemented!()
        }
        fn pattern(&self) -> &dyn Pattern {
            &self.pattern
        }
        fn guard(&self) -> Option<Expr> {
            self.guard.clone()
        }
        fn body(&self) -> Expr {
            syn::parse_quote!(BODY)
        }

        fn span(&self) -> Span {
            Span::call_site()
        }
    }

    // Mock PatternCodeGen trait
    struct MockPatternCodeGen;

    impl PatternCodeGen for MockPatternCodeGen {
        fn generate<P: SubPatternCodeGen>(pattern: &dyn Pattern) -> TokenStream {
            // Just write the number of sub-patterns
            format_ident!("SIZE_{}", pattern.len()).to_token_stream()
        }
    }

    // Mock context
    fn context() -> Context {
        Context {
            incoming_message: parse_quote!(dont_care),
            mailbox: parse_quote!(dont_care),
            message_type: parse_quote!(MSG_TYPE),
        }
    }

    // Unpacking code gen tests
    #[test]
    fn test_unpacking_code_single() {
        let pattern = MockPattern::<1> {};

        let generated = GuardCompiler::unpacking_code(
            Span::call_site(),
            &pattern,
            &format_ident!("msg"),
            &format_ident!("mapping"),
        );

        assert_tokens!(generated, { msg[mapping.get(0usize)] });
    }

    #[test]
    fn test_unpacking_code_triple() {
        let pattern = MockPattern::<3> {};

        let generated = GuardCompiler::unpacking_code(
            Span::call_site(),
            &pattern,
            &format_ident!("msg"),
            &format_ident!("mapping"),
        );

        assert_tokens!(generated, {
            (
                msg[mapping.get(0usize)],
                msg[mapping.get(1usize)],
                msg[mapping.get(2usize)],
            )
        });
    }

    // Guard expression code retrieval
    #[test]
    fn test_guard_is_none_expr() {
        let generated = GuardCompiler::guard_expression_code(Span::call_site(), None);
        assert_tokens!(generated, { true });
    }

    #[test]
    fn test_guard_is_some_expr() {
        let expr: syn::Expr = parse_quote!(x > 5 && y == 52);
        let generated = GuardCompiler::guard_expression_code(Span::call_site(), Some(expr));
        assert_tokens!(generated, { x > 5 && y == 52 });
    }

    // Test GuardCompiler code gen
    #[test]
    fn test_guardcompiler_single_pattern_no_guard() {
        let case = MockCase::<1>::new(None);

        let generated =
            GuardCompiler::generate::<MockPatternCodeGen>(&case, &context(), "guard_fn");

        assert_tokens!(generated, {
            fn guard_fn(
                messages: &[&MSG_TYPE; 1usize],
                mapping: &fairjax_core::Mapping<1usize>,
            ) -> bool {
                match messages[mapping.get(0usize)] {
                    SIZE_1 => true,
                    _ => false,
                }
            }
        });
    }

    #[test]
    fn test_guardcompiler_large_pattern_with_guard() {
        let guard_expr: syn::Expr = parse_quote!(a == 1 && b < 10);
        let case = MockCase::<4>::new(Some(guard_expr));

        let generated =
            GuardCompiler::generate::<MockPatternCodeGen>(&case, &context(), "guard_fn_large");

        assert_tokens!(generated, {
            fn guard_fn_large(
                messages: &[&MSG_TYPE; 4usize],
                mapping: &fairjax_core::Mapping<4usize>,
            ) -> bool {
                match (
                    messages[mapping.get(0usize)],
                    messages[mapping.get(1usize)],
                    messages[mapping.get(2usize)],
                    messages[mapping.get(3usize)],
                ) {
                    SIZE_4 => a == 1 && b < 10,
                    _ => false,
                }
            }
        });
    }
}
