use crate::parse::pattern::PatternDefinition;

#[derive(Clone)]
pub enum Strategy {
    StatefulTree,
    BruteForce,
    Partitions {
        vars: Vec<String>,
        pattern: PatternDefinition,
    },
    // PartitionsGuard { inner_case: CaseDefinition },
}
