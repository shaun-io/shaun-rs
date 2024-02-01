use core::fmt::Debug;

use std::sync::Arc;

use crate::parser::expression::Expression;

#[derive(PartialEq, Debug, Clone)]
pub enum LogicalPlan {
    Projection(Projection),
    Filter(Filter),
    Aggregate(Aggregate),
    Sort,
    Join,
    Statement,
    TableScan,
    Subquery,
    Limit,
    Values,
    Explain,
    Dml,
    Ddl,
}

/// Evaluates an arbitrary list of expressions (essentially a
/// SELECT with an expression list) on its input.
#[derive(PartialEq, Debug, Clone)]
// mark non_exhaustive to encourage use of try_new/new()
#[non_exhaustive]
pub struct Projection {
    /// The list of expressions
    pub expr: Vec<Expression>,
    /// The incoming logical plan
    pub input: Arc<LogicalPlan>,
    /// The schema description of the output
    pub schema: Option<String>,
}

#[derive(PartialEq, Debug, Clone)]
#[non_exhaustive]
pub struct Filter {
    /// The predicate expression, which must have Boolean type.
    pub predicate: Expression,
    /// The incoming logical plan
    pub input: Box<LogicalPlan>,
}

#[derive(PartialEq, Debug, Clone)]
#[non_exhaustive]
pub(crate) struct Aggregate {}
