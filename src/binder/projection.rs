use crate::types::expr::Expr;

use super::logical_plan::LogicalPlan;

#[derive(Debug, Clone)]
pub struct Projection {
    pub input: Box<LogicalPlan>,
    pub exprs: Vec<Expr>,
}
