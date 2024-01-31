use std::sync::Arc;

use crate::parser::stmt::Statement;

use super::logical_operation::LogicalPlan;

pub struct BoundStatement {
    _names: Vec<String>,
    plan: Arc<LogicalPlan>,
}

pub struct Binder {}

impl Binder {
    pub fn create_plan(stmt: Statement) {}
}
