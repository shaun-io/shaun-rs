use crate::types::expr::Expr;

use super::logical_plan::LogicalPlan;

#[derive(Debug, Clone)]
pub struct Aggregate {
    pub input: Box<LogicalPlan>,
    pub group_by_exprs: Vec<Expr>,
    pub aggr_exprs: Vec<Expr>,
}
