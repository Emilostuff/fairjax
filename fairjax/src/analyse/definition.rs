use crate::analyse::bundle::CaseBundleDefinition;
use crate::parse::context::Context;
use crate::parse::definition::RawJoinDefinition;
use syn::Result;

pub struct JoinDefinition {
    pub context: Context,
    pub cases: Vec<CaseBundleDefinition>,
    //pub mappings: Vec<usize>,
}

impl JoinDefinition {
    pub fn analyse(input: RawJoinDefinition) -> Result<Self> {
        Ok(Self {
            context: input.context,
            cases: input
                .cases
                .into_iter()
                .map(CaseBundleDefinition::analyse)
                .collect::<Result<_>>()?,
            //mappings: vec![],
        })
    }
}
