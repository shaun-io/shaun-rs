use super::Planner;
use crate::{
    catalog::{column, TableID},
    error::{
        Error::{Internal, Plan},
        Result,
    },
    fmt_err,
    parser::{
        expression::{Expression, *},
        operation::{self, Operation},
        stmt::ExplainStmt,
    },
    types::{
        self,
        expr::{self, AggExpr, Expr, Operator, SUPPORT_AGG_NAME},
        value::Value,
    },
};

impl Planner {
    pub fn plan_expression(&self, expr: &Expression) -> Result<expr::Expr> {
        match expr {
            Expression::Field(table_name, column_name) => match table_name {
                Some(table_name) => match self.context.info_by_name(&table_name) {
                    Some(info) => {
                        for (idx, col) in info.schema.columns.iter().enumerate() {
                            if (*col).name == (*column_name) {
                                return Ok(Expr::ColumnExpr {
                                    column_index: idx as TableID,
                                    table_name: table_name.clone(),
                                    column_name: column_name.clone(),
                                });
                            }
                        }

                        return Err(Plan(fmt_err!("{column_name} is not exist in {table_name}")));
                    }
                    None => return Err(Plan(fmt_err!("{table_name} is not exist"))),
                },
                None => {
                    // eg: `SELECT * FROM t1 JOIN t2 ON (t1.)column1 = (t2.)column2;`
                    // plan the expression, but don't known the column_name belone.
                    match self.context.tablenames_by_columnname(&column_name) {
                        Some(table_names) => {
                            if table_names.len() == 1 {
                                match self.context.info_by_name(&table_names[0]) {
                                    Some(info) => {
                                        for (idx, col) in info.schema.columns.iter().enumerate() {
                                            if (*col).name == (*column_name) {
                                                return Ok(Expr::ColumnExpr {
                                                    column_index: idx as TableID,
                                                    table_name: table_names[0].clone(),
                                                    column_name: column_name.clone(),
                                                });
                                            }
                                        }
                                    }
                                    None => {
                                        return Err(Internal(fmt_err!(
                                            "{table_names:?} must exist in table_map"
                                        )));
                                    }
                                }
                            } else {
                                return Err(Plan(fmt_err!(
                                    "{column_name} is fuzzy, {table_names:?}"
                                )));
                            }
                        }
                        None => {
                            return Err(Plan(fmt_err!("{column_name} is not exist")));
                        }
                    }

                    todo!()
                }
            },
            Expression::Column(_) => todo!(),
            Expression::Literal(literal) => match literal {
                Literal::All => Ok(Expr::Literal(Value::All)),
                Literal::Null => Ok(Expr::Literal(Value::Null)),
                Literal::Bool(v) => Ok(Expr::Literal(Value::Bool(*v))),
                Literal::Int(v) => Ok(Expr::Literal(Value::Int(*v))),
                Literal::Float(v) => Ok(Expr::Literal(Value::Float(*v))),
                Literal::String(v) => Ok(Expr::Literal(Value::String(v.clone()))),
            },
            Expression::Function(func_name, args) => match func_name.as_str() {
                "MIN" => Ok(Expr::Agg {
                    agg_type: AggExpr::Min,
                    args: self.plan_expression_list(args)?,
                }),
                "MAX" => Ok(Expr::Agg {
                    agg_type: AggExpr::Max,
                    args: self.plan_expression_list(args)?,
                }),
                "COUNT" => Ok(Expr::Agg {
                    agg_type: AggExpr::Count,
                    args: self.plan_expression_list(args)?,
                }),
                "SUM" => Ok(Expr::Agg {
                    agg_type: AggExpr::Sum,
                    args: self.plan_expression_list(args)?,
                }),
                _ => {
                    return Err(Plan(fmt_err!(
                        "{func_name} unknown function_name, only support {SUPPORT_AGG_NAME:#?}"
                    )));
                }
            },
            Expression::Operation(op) => self.plan_operation(op),
        }
    }

    fn plan_binary_op(
        &self,
        l: &Box<Expression>,
        r: &Box<Expression>,
        op: Operator,
    ) -> Result<expr::Expr> {
        Ok(Expr::BinaryExpr {
            left: Box::new(self.plan_expression(&*l)?),
            op,
            right: Box::new(self.plan_expression(&*r)?),
        })
    }

    fn plan_operation(&self, op: &Operation) -> Result<expr::Expr> {
        Ok(match op {
            Operation::And(l, r) => self.plan_binary_op(l, r, Operator::And)?,
            Operation::Not(_) => {
                todo!()
            }
            Operation::Or(l, r) => self.plan_binary_op(l, r, Operator::Or)?,
            Operation::NotEqual(l, r) => self.plan_binary_op(l, r, Operator::NotEq)?,
            Operation::Equal(l, r) => self.plan_binary_op(l, r, Operator::Eq)?,
            Operation::GreaterThan(l, r) => self.plan_binary_op(l, r, Operator::Gt)?,
            Operation::GreaterThanOrEqual(l, r) => self.plan_binary_op(l, r, Operator::GtEq)?,
            Operation::LessThan(l, r) => self.plan_binary_op(l, r, Operator::Lt)?,
            Operation::LessThanOrEqual(l, r) => self.plan_binary_op(l, r, Operator::LtEq)?,
            Operation::IsNull(_) => todo!(),
            Operation::Add(l, r) => self.plan_binary_op(l, r, Operator::Plus)?,
            Operation::Subtract(l, r) => self.plan_binary_op(l, r, Operator::Minus)?,
            Operation::Multiply(l, r) => self.plan_binary_op(l, r, Operator::Multiply)?,
            Operation::Divide(l, r) => self.plan_binary_op(l, r, Operator::Divide)?,
            Operation::Assert(_) => todo!(),
            Operation::Like(..) => todo!(),
            Operation::Negate(_) => todo!(),
            Operation::BitWiseNot(_) => todo!(),
            Operation::Modulo(..) => todo!(),
        })
    }

    fn plan_expression_list(&self, exprs_list: &Vec<Expression>) -> Result<Vec<Expr>> {
        let mut res = vec![];
        for expr in exprs_list {
            res.push(self.plan_expression(expr)?);
        }

        Ok(res)
    }
}
