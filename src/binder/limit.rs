use crate::types::expr::Expr;

use super::logical_plan::LogicalPlan;

#[derive(Debug, Clone)]
pub struct Limit {
    pub input: Box<LogicalPlan>,
    pub limit: u64,
}
