use std::sync::Arc;

use self::{context::PlanContext, logical_plan::PlanNode};
use crate::{catalog::CataLog, parser::stmt::Statement};

mod context;
pub mod logical_plan;
mod plan_expression;
mod plan_select;
mod plan_createtable;
mod optimize;

use crate::error::Result;

#[derive(Debug)]
pub struct Planner {
    pub catalog: Arc<CataLog>,
    pub context: PlanContext,
}

impl Planner {
    pub fn new(catalog: &Arc<CataLog>) -> Self {
        Self {
            catalog: Arc::clone(catalog),
            context: PlanContext::new(),
        }
    }

    /// planner will catch the sql metainfo
    /// use it after reset_ctx
    pub fn reset_ctx(&mut self) -> &mut Self {
        self.context = PlanContext::new();

        self
    }

    pub fn plan(&mut self, ast: &Statement) -> Result<PlanNode> {
        match ast {
            Statement::Select(select_ast) => self.plan_select(&select_ast),
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog::{column::Column, schema::Schema},
        parser::{stmt::JoinType, Parser},
        types::{
            expr::{Expr, Operator},
            value::Value,
            LogicalType,
        },
    };

    #[test]
    fn test_select() {
        let mut parser = Parser::new_parser("".to_owned());
        let mut catalog = CataLog::default();
        catalog
            .create_table(
                &"user".to_owned(),
                &Schema {
                    columns: vec![
                        Column::new(LogicalType::UInt64, "id"),
                        Column::new(LogicalType::String, "user"),
                        Column::new(LogicalType::Int32, "year"),
                        Column::new(LogicalType::UInt16, "account"),
                    ],
                },
            )
            .unwrap();
        catalog
            .create_table(
                &"cccc".to_owned(),
                &Schema {
                    columns: vec![Column::new(LogicalType::UInt32, "id")],
                },
            )
            .unwrap();
        let catalog = Arc::new(catalog);
        let mut planner = Planner::new(&catalog);
        match parser
            .update(r#"select user.id, user from user join cccc on cccc.id = user.id;"#)
            .parse_stmt()
        {
            Ok(ast) => {
                assert_eq!(
                    planner.plan(&ast).unwrap(),
                    PlanNode::Projection {
                        exprs: vec![
                            (
                                Expr::ColumnExpr {
                                    column_index: 0,
                                    table_name: "user".to_owned(),
                                    column_name: "id".to_owned(),
                                },
                                None,
                            ),
                            (
                                Expr::ColumnExpr {
                                    column_index: 1,
                                    table_name: "user".to_owned(),
                                    column_name: "user".to_owned()
                                },
                                None
                            )
                        ],
                        input: Box::new(PlanNode::Join {
                            left: Box::new(PlanNode::Scan {
                                table_name: "user".to_owned(),
                                table_id: 0,
                                predicates: None,
                            }),
                            right: Box::new(PlanNode::Scan {
                                table_name: "cccc".to_owned(),
                                table_id: 1,
                                predicates: None,
                            }),
                            join_type: JoinType::Inner,
                            predicates: Expr::BinaryExpr {
                                left: Box::new(Expr::ColumnExpr {
                                    column_index: 0,
                                    table_name: "cccc".to_owned(),
                                    column_name: "id".to_owned(),
                                }),
                                op: Operator::Eq,
                                right: Box::new(Expr::ColumnExpr {
                                    column_index: 0,
                                    table_name: "user".to_owned(),
                                    column_name: "id".to_owned(),
                                })
                            }
                        }),
                    }
                );
            }
            Err(err_msg) => {
                println!("parse error: {err_msg:?}");
                assert!(false);
            }
        }
        planner.reset_ctx();
        match parser
            .update(
                r#"select user.id, user from user join cccc on cccc.id = user.id where cccc.id > 10"#,
            )
            .parse_stmt()
        {
            Ok(ast) => {
                assert_eq!(
                    planner.plan(&ast).unwrap(),
                    PlanNode::Projection {
                        exprs: vec![
                            (
                                Expr::ColumnExpr {
                                    column_index: 0,
                                    table_name: "user".to_owned(),
                                    column_name: "id".to_owned(),
                                },
                                None,
                            ),
                            (
                                Expr::ColumnExpr {
                                    column_index: 1,
                                    table_name: "user".to_owned(),
                                    column_name: "user".to_owned()
                                },
                                None
                            )
                        ],
                        input: Box::new(PlanNode::Filter {
                            input: Box::new(PlanNode::Join {
                                left: Box::new(PlanNode::Scan {
                                    table_name: "user".to_owned(),
                                    table_id: 0,
                                    predicates: None,
                                }),
                                right: Box::new(PlanNode::Scan {
                                    table_name: "cccc".to_owned(),
                                    table_id: 1,
                                    predicates: None,
                                }),
                                join_type: JoinType::Inner,
                                predicates: Expr::BinaryExpr {
                                    left: Box::new(Expr::ColumnExpr {
                                        column_index: 0,
                                        table_name: "cccc".to_owned(),
                                        column_name: "id".to_owned(),
                                    }),
                                    op: Operator::Eq,
                                    right: Box::new(Expr::ColumnExpr {
                                        column_index: 0,
                                        table_name: "user".to_owned(),
                                        column_name: "id".to_owned(),
                                    })
                                }
                            }),
                            predicates: Expr::BinaryExpr {
                                left: Box::new(Expr::ColumnExpr {
                                    column_index: 0,
                                    table_name: "cccc".to_owned(),
                                    column_name: "id".to_owned()
                                }),
                                op: Operator::Gt,
                                right: Box::new(Expr::Literal(Value::Int(10))),
                            },
                        })
                    }
                );
            }
            Err(err_msg) => {
                println!("parse error: {err_msg:?}");
                assert!(false);
            }
        }
    }

    #[test]
    fn test_multi_join() {

    }
}
