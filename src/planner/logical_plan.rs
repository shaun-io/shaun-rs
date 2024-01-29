use crate::{catalog::{schema::SchemaRef, TableID}, parser::stmt::{JoinType, OrderByType}, types::expr::Expr};


#[derive(Debug, PartialEq)]
pub enum PlanNode {
    Scan {
        table_name: String,
        table_id: TableID,
        predicates: Option<Expr>,
    },
    Aggregation {
        input: Box<PlanNode>,
        schema: SchemaRef,
    },
    Filter {
        input: Box<PlanNode>,
        predicates: Expr,
    },
    Join {
        left: Box<PlanNode>,
        right: Box<PlanNode>,
        join_type: JoinType,
        predicates: Expr,
    },
    Projection {
        exprs: Vec<(Expr, Option<String>)>,
        input: Box<PlanNode>,
    },
    Sort {
        input: Box<PlanNode>,
        by_column: Vec<(Expr, OrderByType)>,
    },
    Null,
}
