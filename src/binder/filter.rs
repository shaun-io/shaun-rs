use crate::types::expr::Expr;

use super::logical_plan::LogicalPlan;

#[derive(Debug, Clone)]
pub struct Filter {
    pub input: Box<LogicalPlan>,
    pub predicate: Expr,
}
