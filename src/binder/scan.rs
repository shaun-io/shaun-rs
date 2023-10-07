use crate::types::expr::Expr;

use super::logical_plan::LogicalPlan;

#[derive(Debug, Clone)]
pub struct Scan {
    pub table_name: String,
    pub source: Box<LogicalPlan>,
    pub projection_filters: Option<Vec<usize>>,
    pub filters: Option<Vec<Expr>>,
    pub fetch: Option<usize>,
}
