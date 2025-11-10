use crate::parse::case::CaseDefinition;
use crate::parse::context::Context;
use proc_macro2::TokenStream;
use syn::Result;

pub struct RawJoinDefinition {
    pub context: Context,
    pub cases: Vec<CaseDefinition>,
}

// Input Parsing
impl RawJoinDefinition {
    pub fn parse(input: TokenStream) -> Result<Self> {
        // Parse tokens to match expression syntax
        let match_expr: syn::ExprMatch = syn::parse2(input)?;

        // Parse context
        let context = Context::parse(*match_expr.expr)?;

        // Parse match arms to cases:
        let cases = match_expr
            .arms
            .into_iter()
            .enumerate()
            .map(|(i, arm)| CaseDefinition::parse(arm, i))
            .collect::<Result<Vec<CaseDefinition>>>()?;

        // Return join defenition
        Ok(RawJoinDefinition { context, cases })
    }
}
