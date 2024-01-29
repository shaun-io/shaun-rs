use std::sync::Arc;

use super::{
    logical_plan::{self, PlanNode},
    Planner,
};
use crate::{
    error::{Error::Plan, Result},
    fmt_err,
    parser::{
        expression::Expression,
        stmt::{self, FromItem, SelectStmt, *},
    },
};

impl Planner {
    // planbuilder.go:243
    /// if sql has not `from` item, there are 4 examples for this:
    /// ```
    /// SELECT function_name(args);
    /// SELECT constant_expr;
    /// SELECT @vars;
    /// and because we don't support subquery,
    /// SELECT (SELECT ...) is error.
    /// ```
    pub(crate) fn plan_select(&mut self, select_stmt: &SelectStmt) -> Result<PlanNode> {
        // let node = match &select_stmt.froms {
        //     Some(froms) => self.plan_from(&froms[0]).unwrap(),
        //     None => PlanNode::Null,
        // };

        // let node = match &select_stmt.wheres {
        //     Some(where_expr) => PlanNode::Filter(Filter {
        //         source: Box::new(node),
        //         predicate: self.plan_expression(where_expr)?,
        //     }),
        //     None => PlanNode::Null,
        // };
        // // in shaun-parser, actually here is only a group by expr

        // let mut is_has_agg = false;
        // for (expr, _) in &select_stmt.selects {
        //     if expr.has_aggregation()? {
        //         is_has_agg = true;
        //         break;
        //     }
        // }
        if select_stmt.selects.is_empty() {
            return Err(Plan("can't select empty".to_owned()));
        }
        let node = match &select_stmt.froms {
            Some(froms) => self.plan_from(&froms[0])?,
            None => {
                // if the 'from' is empty
                // it must like:
                //   1. `SELECT constant_expresssion;`
                //   2. `SELECT @variable_name;`
                todo!()
            }
        };
        let node = match &select_stmt.wheres {
            Some(where_expr) => PlanNode::Filter {
                input: Box::new(node),
                predicates: self.plan_expression(where_expr)?,
            },
            None => node,
        };

        let mut is_has_agg = false;
        for (expr, _) in &select_stmt.selects {
            if expr.has_aggregation()? {
                is_has_agg = true;
                break;
            }
        }
        let node = if is_has_agg || select_stmt.group_by.is_some() {
            todo!()
        } else {
            self.plan_normal_select(select_stmt, node)?
        };

        let node = match &select_stmt.order {
            Some(order_by) => PlanNode::Sort {
                input: Box::new(node),
                by_column: {
                    let mut exprs = vec![];
                    for (expr, orderby_type) in order_by {
                        exprs.push((self.plan_expression(&expr)?, orderby_type.clone()));
                    }
                    exprs
                },
            },
            None => node,
        };

        Ok(node)
    }

    pub(crate) fn plan_normal_select(
        &mut self,
        select_stmt: &SelectStmt,
        node: PlanNode,
    ) -> Result<PlanNode> {
        Ok(PlanNode::Projection {
            exprs: {
                let mut exprs = vec![];
                for (expr, alias) in &select_stmt.selects {
                    exprs.push((self.plan_expression(&expr)?, alias.to_owned()));
                }

                exprs
            },
            input: Box::new(node),
        })
    }

    /// for example, we have this SQL:
    /// ```
    /// SELECT v3, MAX(v1) + MAX(v2)
    /// FROM user
    /// WHERE condition_expression
    /// GROUP BY v3
    /// HAVING COUNT(v4) > COUNT(v5);
    /// ```
    /// we hope generate this logical plan:
    /// ```
    ///  Projection([v3, MAX(v1) + MAX(v2)])
    ///   GroupBy(v3) agg_types = [MAX(v1), MAX(v2), COUNT(v4), COUNT(v5)]
    ///    Filter(condition_expression)
    ///     TableScan(user)
    /// ```
    /// and actuall here is only has a GroupBy expresssion
    pub(crate) fn plan_select_agg(
        &mut self,
        select_stmt: &SelectStmt,
        node: &mut PlanNode,
    ) -> Result<PlanNode> {
        let mut group_by_exprs = vec![];
        match &select_stmt.group_by {
            Some(exprs) => {
                group_by_exprs.push(self.plan_expression(&exprs[0])?);
            }
            None => {}
        };

        todo!()
    }

    pub(crate) fn plan_from(&mut self, join_item: &FromItem) -> Result<PlanNode> {
        Ok(match join_item {
            FromItem::Join {
                left,
                right,
                join_type,
                predicate,
            } => {
                let left_node = self.plan_from(left)?;
                let right_node = self.plan_from(right)?;

                // in mysql, if don't write the join condition in SQL, the default way
                // is join with two tables primary key.
                // but here if you write this `SELECT * FROM t1 JOIN t2;` It's an error
                match predicate {
                    Some(expr) => PlanNode::Join {
                        left: Box::new(left_node),
                        right: Box::new(right_node),
                        join_type: join_type.clone(),
                        predicates: self.plan_expression(expr)?,
                    },
                    None => {
                        return Err(Plan(fmt_err!("join must take with condition expression")));
                    }
                }
            }
            FromItem::Table { name, alias } => PlanNode::Scan {
                table_name: name.clone(),
                table_id: match self.catalog.table_by_name(name) {
                    Some(info) => {
                        self.context.insert_table_info(name, alias, &info)?;

                        info.id
                    }
                    None => {
                        return Err(Plan(fmt_err!("table: {name} is not exist")));
                    }
                },
                predicates: None,
            },
        })
    }
}
