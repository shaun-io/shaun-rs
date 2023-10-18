use crate::{parser::stmt::JoinType, types::expr::Expr};

use super::logical_plan::LogicalPlan;

#[derive(Clone, Debug)]
pub struct Join {
    pub left_input: Box<LogicalPlan>,
    pub right_input: Box<LogicalPlan>,
    pub join_type: JoinType,
    pub filter: Option<Expr>,
}
