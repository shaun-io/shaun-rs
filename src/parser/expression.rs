use crate::parser::operation::Operation;

#[derive(PartialEq, Debug, Clone)]
// 字面量
pub enum Literal {
    All,
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expression {
    Field(Option<String>, String),
    Column(usize),
    Literal(Literal),
    Function(String, Vec<Expression>),
    Operation(Operation),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::stmt::*;
    use crate::parser::test::init;
    use crate::parser::Parser;
    use log::error;

    #[test]
    fn parse_expression_test() {
        init();

        // Token::Number("123") Token::Plus Token::Number("456");
        let mut parser = Parser::new_parser("SELECT 123 + 456;".to_owned());
        let result_exp = Expression::Operation(Operation::Add(
            Box::new(Expression::Literal(Literal::Int(123))),
            Box::new(Expression::Literal(Literal::Int(456))),
        ));
        let mut expr_selects = vec![];
        expr_selects.push((result_exp.clone(), None));
        let result = Statement::Select(SelectStmt {
            selects: expr_selects,
            froms: None,
            wheres: None,
            group_by: None,
            having: None,
            order: None,
            offset: None,
            limit: None,
        });
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {:?} but get: {:?}", result, e);
                assert!(false);
            }
        };

        parser.update("SELECT 123 + 456 AS c1");
        let mut expr_selects = vec![];
        expr_selects.push((result_exp.clone(), Some("c1".to_owned())));
        let result = Statement::Select(SelectStmt {
            selects: expr_selects,
            froms: None,
            wheres: None,
            group_by: None,
            having: None,
            order: None,
            offset: None,
            limit: None,
        });
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {:?} but get: {:?}", result, e);
                assert!(false);
            }
        }

        parser.update("SELECT 123 + 456 c1");
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {:?} but get: {:?}", result, e);
                assert!(false);
            }
        }

        //            -
        //         +      3
        //      *      /
        //   *    3 456  4
        // 1   2
        parser.update("SELECT 1 * 2 * 3 + 456 / 4 - 3 c1;");
        let mut res_expr = Expression::Operation(Operation::Subtract(
            Box::new(Expression::Operation(Operation::Add(
                Box::new(Expression::Operation(Operation::Multiply(
                    Box::new(Expression::Operation(Operation::Multiply(
                        Box::new(Expression::Literal(Literal::Int(1))),
                        Box::new(Expression::Literal(Literal::Int(2))),
                    ))),
                    Box::new(Expression::Literal(Literal::Int(3))),
                ))),
                Box::new(Expression::Operation(Operation::Divide(
                    Box::new(Expression::Literal(Literal::Int(456))),
                    Box::new(Expression::Literal(Literal::Int(4))),
                ))),
            ))),
            Box::new(Expression::Literal(Literal::Int(3))),
        ));
        let mut expr_selects = vec![];
        expr_selects.push((res_expr.clone(), Some("c1".to_owned())));
        let result = Statement::Select(SelectStmt {
            selects: expr_selects.clone(),
            froms: None,
            wheres: None,
            group_by: None,
            having: None,
            order: None,
            offset: None,
            limit: None,
        });
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {:?} but get: {:?}", result, e);
                assert!(false);
            }
        }

        // parse bool expression
        parser.update("SELECT 1 >= 10");
        res_expr = Expression::Operation(Operation::GreaterThanOrEqual(
            Box::new(Expression::Literal(Literal::Int(1))),
            Box::new(Expression::Literal(Literal::Int(10))),
        ));
        expr_selects.clear();
        expr_selects.push((res_expr, None));
        let result = Statement::Select(SelectStmt {
            selects: expr_selects.clone(),
            froms: None,
            wheres: None,
            group_by: None,
            having: None,
            order: None,
            offset: None,
            limit: None,
        });
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {:?} but get: {:?}", result, e);
                assert!(false);
            }
        }
        //
        parser.update("SELECT (1 <= 10);");
        res_expr = Expression::Operation(Operation::LessThanOrEqual(
            Box::new(Expression::Literal(Literal::Int(1))),
            Box::new(Expression::Literal(Literal::Int(10))),
        ));
        expr_selects.clear();
        expr_selects.push((res_expr, None));
        let result = Statement::Select(SelectStmt {
            selects: expr_selects.clone(),
            froms: None,
            wheres: None,
            group_by: None,
            having: None,
            order: None,
            offset: None,
            limit: None,
        });
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {:?} but get: {:?}", result, e);
                assert!(false);
            }
        }

        parser.update("SELECT (1 <= 10) AND (1 >= 10.1);");
        res_expr = Expression::Operation(Operation::And(
            Box::new(Expression::Operation(Operation::LessThanOrEqual(
                Box::new(Expression::Literal(Literal::Int(1))),
                Box::new(Expression::Literal(Literal::Int(10))),
            ))),
            Box::new(Expression::Operation(Operation::GreaterThanOrEqual(
                Box::new(Expression::Literal(Literal::Int(1))),
                Box::new(Expression::Literal(Literal::Float(10.1))),
            ))),
        ));
        expr_selects.clear();
        expr_selects.push((res_expr, None));
        let result = Statement::Select(SelectStmt {
            selects: expr_selects.clone(),
            froms: None,
            wheres: None,
            group_by: None,
            having: None,
            order: None,
            offset: None,
            limit: None,
        });
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {:?} but get: {:?}", result, e);
                assert!(false);
            }
        }

        // 测试 selects Comma是否正确
        let result = Statement::Select(SelectStmt {
            selects: vec![
                (
                    Expression::Operation(Operation::Add(
                        Box::new(Expression::Literal(Literal::Int(1))),
                        Box::new(Expression::Literal(Literal::Int(2))),
                    )),
                    None,
                ),
                (
                    Expression::Operation(Operation::Divide(
                        Box::new(Expression::Literal(Literal::Float(10.1))),
                        Box::new(Expression::Literal(Literal::Bool(false))),
                    )),
                    None,
                ),
            ],
            froms: None,
            wheres: None,
            group_by: None,
            having: None,
            order: None,
            offset: None,
            limit: None,
        });
        match parser.update("SELECT 1 + 2, 10.1 / FALSE;").parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {:?} but get: {:?}", result, e);
                assert!(false);
            }
        }
        let result = Statement::Select(SelectStmt {
            selects: vec![
                (
                    Expression::Operation(Operation::Add(
                        Box::new(Expression::Literal(Literal::Int(1))),
                        Box::new(Expression::Literal(Literal::Int(2))),
                    )),
                    None,
                ),
                (
                    Expression::Operation(Operation::Divide(
                        Box::new(Expression::Literal(Literal::Float(10.1))),
                        Box::new(Expression::Literal(Literal::Bool(false))),
                    )),
                    None,
                ),
                (
                    Expression::Operation(Operation::And(
                        Box::new(Expression::Literal(Literal::Bool(true))),
                        Box::new(Expression::Literal(Literal::Float(10.1))),
                    )),
                    None,
                ),
            ],
            froms: None,
            wheres: None,
            group_by: None,
            having: None,
            order: None,
            offset: None,
            limit: None,
        });
        match parser
            .update("SELECT 1 + 2, 10.1 / FALSE, TRUE AND 10.1;")
            .parse_stmt()
        {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {:?} but get: {:?}", result, e);
                assert!(false);
            }
        }

        let result = Statement::Select(SelectStmt {
            selects: vec![
                (
                    Expression::Operation(Operation::Add(
                        Box::new(Expression::Literal(Literal::Int(1))),
                        Box::new(Expression::Literal(Literal::Int(2))),
                    )),
                    None,
                ),
                (
                    Expression::Operation(Operation::Divide(
                        Box::new(Expression::Literal(Literal::Float(10.1))),
                        Box::new(Expression::Literal(Literal::Bool(false))),
                    )),
                    None,
                ),
                (
                    Expression::Operation(Operation::And(
                        Box::new(Expression::Literal(Literal::Bool(true))),
                        Box::new(Expression::Literal(Literal::Float(10.1))),
                    )),
                    None,
                ),
            ],
            froms: None,
            wheres: None,
            group_by: None,
            having: None,
            order: None,
            offset: None,
            limit: None,
        });
        match parser
            .update("SELECT (1) + (2), ((10.1) / (FALSE)), (TRUE AND 10.1);")
            .parse_stmt()
        {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {:?} but get: {:?}", result, e);
                assert!(false);
            }
        }
    }
}
