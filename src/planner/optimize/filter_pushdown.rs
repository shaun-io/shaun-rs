use super::LogicalOptimizer;
use crate::{planner::logical_plan::PlanNode, types::expr::Expr};

impl LogicalOptimizer {
    pub fn filter_pushdown(&self, node: &PlanNode, predicate: Option<Expr>) {
        match node {
            PlanNode::Projection { input, exprs: _ } => {
                self.filter_pushdown(input, None);
            }
            PlanNode::Filter { input, predicates } => {
                self.filter_pushdown(input, Some(predicates.clone()));
            }
            PlanNode::Join {
                left,
                right,
                join_type,
                predicates,
            } => {
                
            }
            _ => return,
        }
    }
}
