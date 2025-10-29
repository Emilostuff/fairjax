use crate::parse::case::{Case, CaseDefinition};
use crate::parse::context::Context;
use proc_macro2::TokenStream;
use syn::Result;

pub trait Definition {
    fn context(&self) -> Context;
    fn cases(&self) -> Vec<&dyn Case>;
}

impl Definition for JoinDefinition {
    fn context(&self) -> Context {
        self.context.clone()
    }

    fn cases(&self) -> Vec<&dyn Case> {
        self.cases.iter().map(|c| c as &dyn Case).collect()
    }
}

pub struct JoinDefinition {
    pub context: Context,
    pub cases: Vec<CaseDefinition>,
}

// Input Parsing
impl JoinDefinition {
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
        Ok(JoinDefinition { context, cases })
    }
}
