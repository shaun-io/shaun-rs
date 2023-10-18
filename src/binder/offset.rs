use crate::types::expr::Expr;

use super::logical_plan::LogicalPlan;

#[derive(Debug, Clone)]
pub struct Offset {
    pub input: Box<LogicalPlan>,
    pub offset: u64,
}
