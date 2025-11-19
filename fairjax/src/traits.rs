use crate::analyse::groups::SubPatternGroups;
use crate::analyse::partition::Partitioning;
use crate::analyse::strategy::Strategy;
use crate::parse::context::Context;
use crate::parse::pattern::PatternDefinition;
use crate::parse::sub_pattern::SubPatternDefinition;
use proc_macro2::Span;
use syn::{Expr, Ident};

pub trait Definition {
    fn context(&self) -> Context;
    fn cases(&self) -> Vec<&dyn CaseBundle>;
}

impl Definition for crate::analyse::definition::JoinDefinition {
    fn context(&self) -> Context {
        self.context.clone()
    }

    fn cases(&self) -> Vec<&dyn CaseBundle> {
        self.cases.iter().map(|cb| cb as &dyn CaseBundle).collect()
    }
}

pub trait CaseBundle {
    fn case(&self) -> &dyn Case;
    fn strategy(&self) -> &Strategy;
    fn partitioning(&self) -> &Option<Partitioning>;
    fn sub_pattern_groups(&self) -> &SubPatternGroups;
    fn sub_pattern_at_index(&self, index: usize) -> &dyn SubPattern;
}

impl CaseBundle for crate::analyse::bundle::CaseBundleDefinition {
    fn case(&self) -> &dyn Case {
        &self.case as &dyn Case
    }

    fn strategy(&self) -> &Strategy {
        &self.strategy
    }

    fn partitioning(&self) -> &Option<Partitioning> {
        &self.partitioning
    }

    fn sub_pattern_groups(&self) -> &SubPatternGroups {
        &self.groups
    }

    fn sub_pattern_at_index(&self, index: usize) -> &dyn SubPattern {
        self.case
            .pattern
            .sub_patterns
            .get(index)
            .expect("index provided must be valid")
    }
}

pub trait Case {
    fn index(&self) -> usize;
    fn pattern(&self) -> &dyn Pattern;
    fn guard(&self) -> Option<Expr>;
    fn body(&self) -> Expr;
    fn span(&self) -> Span;
    fn ident_with_case_id(&self, name: &'static str) -> Ident;
}

impl Case for crate::parse::case::CaseDefinition {
    fn index(&self) -> usize {
        self.index
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

    fn span(&self) -> Span {
        self.span
    }
    fn ident_with_case_id(&self, name: &'static str) -> Ident {
        Ident::new(&format!("{}{}", name, self.index()), self.span())
    }
}

pub trait Pattern {
    fn sub_patterns(&self) -> Vec<&dyn SubPattern>;
    fn len(&self) -> usize;
    fn span(&self) -> Span;
}

impl Pattern for PatternDefinition {
    fn sub_patterns(&self) -> Vec<&dyn SubPattern> {
        self.sub_patterns
            .iter()
            .map(|sp| sp as &dyn SubPattern)
            .collect()
    }

    fn len(&self) -> usize {
        self.sub_patterns.len()
    }

    fn span(&self) -> Span {
        self.span
    }
}

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
