use crate::types::expr::Expr;

use super::logical_plan::LogicalPlan;

#[derive(Clone, Debug)]
pub struct Sort {
    pub input: Box<LogicalPlan>,
    pub sort_exprs: Vec<Expr>,
}
