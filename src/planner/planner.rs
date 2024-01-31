use crate::parser::stmt::Statement;

use super::logical_operation::LogicalPlan;

pub struct Planner {
    logic_plan: Option<LogicalPlan>,
}

impl Planner {
    pub fn create_plan(&mut self, _stmt: Statement) {
        self.logic_plan = None;
    }
}
