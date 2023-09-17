mod column;
mod data_type;
mod expression;
mod keyword;
pub mod lexer;
mod operation;
mod operator;
mod stmt;
pub mod token;

use crate::parser::operator::{is_infix_oper, is_prefix_oper};
use crate::parser::{operation::Operation, operator::match_precedence};

use crate::{
    error::{Error, Result},
    fmt_err,
};
use data_type::DataType;
use expression::Expression;
use expression::Literal;
use keyword::Keyword;
use lexer::Lexer;
use log::debug;
use stmt::Statement;
use token::Token;

use self::{
    column::Column,
    operator::Precedence,
    stmt::{FromItem, JoinType, OrderByType, SelectStmt},
};

pub struct Parser {
    lexer: lexer::Lexer,
    pre_token: token::Token,
    peek_token: token::Token,
}

impl Parser {
    pub fn new_parser(sql_str: String) -> Self {
        let mut p = Parser {
            lexer: Lexer::new_lexer(sql_str),
            pre_token: token::Token::Eof,
            peek_token: token::Token::Eof,
        };
        p.peek_token = p.lexer.next_token();
        p.pre_token = p.peek_token.clone();
        p.peek_token = p.lexer.next_token();

        p
    }

    pub fn update(&mut self, sql_str: &str) {
        self.lexer.update(sql_str.to_owned());
        self.peek_token = self.lexer.next_token();
        self.pre_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    pub fn parse_stmt(&mut self) -> Result<Statement> {
        // 直接与 lexer 产生的第一个 Token 作比较
        if self.pre_token == Token::Eof {
            return Err(Error::ParseErr(fmt_err!("empty token {}", self.pre_token)));
        }

        let result = match &self.pre_token {
            Token::KeyWord(Keyword::Begin) => self.parse_transaction_stmt(),
            Token::KeyWord(Keyword::Commit) => self.parse_transaction_stmt(),
            Token::KeyWord(Keyword::Rollback) => self.parse_transaction_stmt(),

            Token::KeyWord(Keyword::Create) => self.parse_create_stmt(),
            Token::KeyWord(Keyword::Drop) => self.parse_drop_stmt(),

            Token::KeyWord(Keyword::Delete) => self.parse_delete_stmt(),
            Token::KeyWord(Keyword::Insert) => self.parse_insert_stmt(),
            Token::KeyWord(Keyword::Select) => self.parse_select_stmt(),
            Token::KeyWord(Keyword::Update) => self.parse_update_stmt(),

            Token::KeyWord(Keyword::Explain) => self.parse_explain_stmt(),

            t => Err(Error::ParseErr(fmt_err!("unexpected token: {}", t))),
        };

        result
    }

    fn parse_transaction_stmt(&mut self) -> Result<Statement> {
        match &self.pre_token {
            // BEGIN TRANSACTION READ ONLY / WRITE;
            // BEGIN TRANSACTION READ ONLY AS OF SYSTEM TIME TimeStamp(u64);
            Token::KeyWord(Keyword::Begin) => {
                let mut is_readonly = false;
                let mut version = None;

                self.next_if_keyword(Keyword::Transaction);

                if self.next_if_token(Token::KeyWord(Keyword::Read)) {
                    match self.next_token() {
                        Token::KeyWord(Keyword::Only) => is_readonly = true,
                        Token::KeyWord(Keyword::Write) => is_readonly = false,

                        t => return Err(Error::ParseErr(fmt_err!("unexpected token: {}", t))),
                    }
                }

                if self.next_if_keyword(Keyword::As) {
                    self.next_expected_keyword(Keyword::Of)?;
                    self.next_expected_keyword(Keyword::System)?;
                    self.next_expected_keyword(Keyword::Time)?;

                    match self.next_token() {
                        Token::Number(n) => version = n.parse::<u64>().ok(),
                        t => {
                            return Err(Error::ParseErr(fmt_err!(
                                "unexpected token: {} expected: Number",
                                t
                            )));
                        }
                    }
                }

                Ok(Statement::Begin(stmt::BeginStmt {
                    is_readonly: is_readonly,
                    version: version,
                }))
            }
            Token::KeyWord(Keyword::Commit) => {
                self.next_token();
                Ok(Statement::Commit)
            }
            Token::KeyWord(Keyword::Rollback) => {
                self.next_token();
                Ok(Statement::Rollback)
            }

            t => Err(Error::ParseErr(fmt_err!("unexpected token: {}", t))),
        }
    }

    #[allow(dead_code)]
    fn parse_ddl_stmt(&mut self) -> Result<Statement> {
        unimplemented!()
    }

    fn parse_delete_stmt(&mut self) -> Result<Statement> {
        unimplemented!()
    }

    fn parse_insert_stmt(&mut self) -> Result<Statement> {
        unimplemented!()
    }

    fn parse_create_stmt(&mut self) -> Result<Statement> {
        // CREATE TABLE table_name
        //  (xxx_name xxx_addr xxx_addr xxx_addr,
        //  xxx, xxx);;
        self.next_expected_keyword(Keyword::Table)?;
        let name = self.next_ident();
        let table_name;
        match name {
            Ok(n) => {
                table_name = n;
            }
            Err(e) => {
                return Err(e);
            }
        }
        self.next_expected_token(Token::LeftParen)?;

        let mut columns = Vec::new();
        loop {
            columns.push(self.parse_column()?);
            let token = self.next_token();

            match token {
                Token::Comma => continue,
                Token::RightParen => break,
                _ => {
                    return Err(Error::ParseErr(fmt_err!(
                        "unexpected token {:?}, want Comma or RightParen",
                        token
                    )));
                }
            }
        }

        self.next_expected_token(Token::Semicolon)?;

        Ok(Statement::CreateTable(stmt::CreateTableStmt {
            columns,
            table_name,
        }))
    }

    fn parse_column(&mut self) -> Result<Column> {
        let name = self.next_ident();
        let column_name;

        match name {
            Ok(n) => {
                column_name = n;
            }
            Err(e) => {
                return Err(e);
            }
        }

        let _column = column::Column {
            name: column_name,
            data_type: match self.next_token() {
                Token::KeyWord(Keyword::Bool) => DataType::Bool,
                Token::KeyWord(Keyword::Boolean) => DataType::Bool,

                Token::KeyWord(Keyword::Float) => DataType::Float,
                Token::KeyWord(Keyword::Double) => DataType::Float,

                Token::KeyWord(Keyword::Int) => DataType::Int,
                Token::KeyWord(Keyword::Integer) => DataType::Int,

                Token::KeyWord(Keyword::Text) => DataType::String,
                Token::KeyWord(Keyword::VarChar) => DataType::String,
                Token::KeyWord(Keyword::Char) => DataType::String,
                Token::KeyWord(Keyword::String) => DataType::String,

                t => {
                    debug!("unexpected token: {}", t);
                    return Err(Error::ParseErr(fmt_err!("unexpected token: {}", t)));
                }
            },
            primary_key: false,
            nullable: None,
            default: None,
            unique: false,
            index: false,
            references: None,
        };
        Ok(_column)
    }

    fn parse_drop_stmt(&mut self) -> Result<Statement> {
        unimplemented!()
    }

    fn parse_select_stmt(&mut self) -> Result<Statement> {
        Ok(Statement::Select(SelectStmt {
            selects: self.parse_clause_select()?,
            froms: self.parse_clause_from()?,
            wheres: self.parse_clause_where()?,
            group_by: match self.parse_clause_group_by() {
                Ok(group_by_expr) => Some(group_by_expr),
                Err(_) => None,
            },
            having: self.parse_clause_having()?,
            order: self.parse_clause_order()?,
            limit: {
                match &self.peek_token {
                    Token::KeyWord(Keyword::Limit) => {
                        self.next_token();
                        self.next_token();
                        Some(match self.parse_expression(Precedence::Lowest)? {
                            Some(exp) => exp,
                            None => {
                                return Err(Error::ParseErr(fmt_err!(
                                    "LIMIT exp should't be none"
                                )));
                            }
                        })
                    }
                    _ => None,
                }
            },
            offset: {
                match &self.peek_token {
                    Token::KeyWord(Keyword::Offset) => {
                        self.next_token();
                        self.next_token();
                        Some(match self.parse_expression(Precedence::Lowest)? {
                            Some(exp) => exp,
                            None => {
                                return Err(Error::ParseErr(fmt_err!(
                                    "OFFSET exp should't be none"
                                )));
                            }
                        })
                    }
                    _ => None,
                }
            },
        }))
    }

    fn parse_clause_select(&mut self) -> Result<Vec<(Expression, Option<String>)>> {
        // SELECT   1 + 3       AS   c1;
        //        [expression]     [alias]
        let mut selects = Vec::new();
        loop {
            if self.next_if_token(Token::Asterisk) && selects.is_empty() {
                break;
            }

            let expr = self.parse_expression(Precedence::Lowest)?.unwrap();
            // SELECT 1 + 2 AS c1; 1 + 2 是一个表达式, c1 是 alias 的一个名字
            // Keyword::As 是一个可选项

            let alias = match self.peek_token.clone() {
                Token::KeyWord(Keyword::As) => {
                    self.next_token();
                    match self.peek_token.clone() {
                        Token::Ident(ident) => {
                            self.next_token();
                            Some(ident)
                        }
                        _ => {
                            return Err(Error::ParseErr(fmt_err!("AS is not valid")));
                        }
                    }
                }
                Token::Ident(ident) => Some(ident),
                _ => None,
            };

            selects.push((expr, alias));

            if !self.next_if_token(Token::Comma) {
                break;
            }
        }

        Ok(selects)
    }

    fn parse_clause_from(&mut self) -> Result<Vec<FromItem>> {
        let mut froms = Vec::new();

        // select expression_list FROM
        match &self.pre_token {
            Token::KeyWord(Keyword::From) => {}
            _ => {
                return Ok(froms);
            }
        }

        loop {
            // FROM table_name as alias_table_name
            let mut item = self.parse_clause_from_table()?;
            self.next_token();
            loop {
                // SELECT t1.xxx, t2.xxx FROM t1
                //   LEFT JOIN t2 ON t1.xxx = t2.xxx;
                let join_type = self.parse_clause_from_jointype()?;
                if join_type.is_none() {
                    break;
                }
                let join_type = join_type.unwrap();

                let left_exp = Box::new(item);
                let right_exp = Box::new(self.parse_clause_from_table()?);
                // 谓词, On 之后的条件,
                let predicate = match join_type {
                    JoinType::Outer => None,
                    _ => {
                        self.next_expected_keyword(Keyword::On)?;
                        self.next_token();

                        Some(self.parse_expression(Precedence::Lowest)?.unwrap())
                    }
                };

                item = FromItem::Join {
                    left: left_exp,
                    right: right_exp,
                    join_type: join_type,
                    predicate,
                };
            }
            froms.push(item);

            if !self.next_if_token(Token::Comma) {
                break;
            }
        }

        Ok(froms)
    }

    fn parse_clause_from_table(&mut self) -> Result<FromItem> {
        let name = self.next_ident()?;

        let alias = match self.peek_token.clone() {
            Token::KeyWord(Keyword::As) => {
                self.next_token();
                match self.peek_token.clone() {
                    Token::Ident(ident) => {
                        self.next_token();
                        Some(ident)
                    }
                    _ => {
                        return Err(Error::ParseErr(fmt_err!("FROM AS is not valid!")));
                    }
                }
            }
            Token::Ident(ident) => Some(ident),
            _ => None,
        };
        Ok(FromItem::Table { name, alias })
    }

    fn parse_clause_from_jointype(&mut self) -> Result<Option<JoinType>> {
        match &self.pre_token {
            Token::KeyWord(Keyword::Outer) => match self.peek_token.clone() {
                Token::KeyWord(Keyword::Join) => {
                    self.next_token();
                    return Ok(Some(JoinType::Outer));
                }
                _ => Ok(None),
            },
            Token::KeyWord(Keyword::Inner) => match self.peek_token.clone() {
                Token::KeyWord(Keyword::Join) => {
                    self.next_token();
                    return Ok(Some(JoinType::Inner));
                }
                _ => Ok(None),
            },
            Token::KeyWord(Keyword::Left) => match self.peek_token.clone() {
                Token::KeyWord(Keyword::Outer) => {
                    self.next_token();
                    match self.peek_token.clone() {
                        Token::KeyWord(Keyword::Join) => {
                            self.next_token();
                            return Ok(Some(JoinType::Left));
                        }
                        _ => Ok(None),
                    }
                }
                Token::KeyWord(Keyword::Join) => {
                    self.next_token();
                    Ok(Some(JoinType::Left))
                }
                _ => Ok(None),
            },
            Token::KeyWord(Keyword::Right) => {
                self.next_token();
                match self.peek_token.clone() {
                    Token::KeyWord(Keyword::Outer) => {
                        self.next_token();
                        match self.peek_token.clone() {
                            Token::KeyWord(Keyword::Join) => {
                                self.next_token();
                                return Ok(Some(JoinType::Right));
                            }
                            _ => Ok(None),
                        }
                    }
                    Token::KeyWord(Keyword::Join) => {
                        self.next_token();
                        Ok(Some(JoinType::Right))
                    }
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    fn parse_clause_group_by(&mut self) -> Result<Vec<Expression>> {
        let mut exprs = Vec::new();

        match &self.peek_token {
            Token::KeyWord(Keyword::Group) => {
                self.next_token();
            }
            _ => {
                return Ok(exprs);
            }
        }

        self.next_expected_keyword(Keyword::By)?;
        self.next_token();

        loop {
            exprs.push(match self.parse_expression(Precedence::Lowest)? {
                Some(e) => e,
                None => {
                    return Err(Error::ParseErr("GROUP BY exp should't be none".to_owned()));
                }
            });

            if !self.next_if_token(Token::Comma) {
                break;
            }
        }

        Ok(exprs)
    }

    fn parse_clause_where(&mut self) -> Result<Option<Expression>> {
        match &self.pre_token {
            Token::KeyWord(Keyword::Where) => {}
            _ => {
                return Ok(None);
            }
        }
        self.next_token();

        return Ok(Some(match self.parse_expression(Precedence::Lowest)? {
            Some(exp) => exp,
            None => {
                return Err(Error::ParseErr(fmt_err!("WHERE exp should't be none")));
            }
        }));
    }

    fn parse_clause_having(&mut self) -> Result<Option<Expression>> {
        match self.pre_token {
            Token::KeyWord(Keyword::Having) => {
                self.next_token();
                Ok(Some(match self.parse_expression(Precedence::Lowest)? {
                    Some(exp) => exp,
                    None => {
                        return Err(Error::ParseErr("HAVING exp should't be none".to_owned()));
                    }
                }))
            }
            _ => Ok(None),
        }
    }

    fn parse_clause_order(&mut self) -> Result<Vec<(Expression, OrderByType)>> {
        match &self.peek_token {
            Token::KeyWord(Keyword::Order) => {}
            _ => {
                return Ok(Vec::new());
            }
        }
        self.next_token();
        self.next_expected_keyword(Keyword::By)?;
        self.next_token();
        let mut orders = Vec::new();

        loop {
            orders.push((
                match self.parse_expression(Precedence::Lowest)? {
                    Some(exp) => exp,
                    None => {
                        return Err(Error::ParseErr(fmt_err!("ORDER BY exp should't be none")));
                    }
                },
                if self.next_if_keyword(Keyword::Asc) {
                    OrderByType::Asc
                } else if self.next_if_keyword(Keyword::Desc) {
                    OrderByType::Desc
                } else {
                    OrderByType::Asc
                },
            ));
            if !self.next_if_token(Token::Comma) {
                break;
            }
        }

        Ok(orders)
    }

    fn parse_update_stmt(&mut self) -> Result<Statement> {
        unimplemented!()
    }
    fn parse_explain_stmt(&mut self) -> Result<Statement> {
        unimplemented!()
    }

    fn next_token(&mut self) -> &Token {
        self.pre_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();

        &self.pre_token
    }

    fn next_if_token(&mut self, t: Token) -> bool {
        *self.next_token() == t
    }

    fn peek_if_token(&mut self, t: Token) -> bool {
        if self.peek_token == t {
            self.next_token();
            return true;
        }

        false
    }

    fn next_if_keyword(&mut self, k: Keyword) -> bool {
        *self.next_token() == Token::KeyWord(k)
    }

    fn next_expected_keyword(&mut self, k: Keyword) -> Result<()> {
        let t = self.next_token();

        if *t == Token::KeyWord(k) {
            Ok(())
        } else {
            Err(Error::ParseErr(fmt_err!(
                "unexpected keyword: {} want: {}",
                t,
                k
            )))
        }
    }

    fn next_expected_token(&mut self, t: Token) -> Result<()> {
        let token = self.next_token();

        if *token == t {
            Ok(())
        } else {
            Err(Error::ParseErr(fmt_err!(
                "unexpected token: {} want: {}",
                token,
                t
            )))
        }
    }

    fn next_ident(&mut self) -> Result<String> {
        match self.next_token() {
            Token::Ident(ident) => Ok(ident.clone()),
            t => Err(Error::ParseErr(fmt_err!(
                "expected: Token::Ident but get: {}",
                t
            ))),
        }
    }

    // (1 + 2)
    fn parse_expression(&mut self, precedence: Precedence) -> Result<Option<Expression>> {
        if !is_prefix_oper(&self.pre_token) {
            dbg!("No prefixOperatorFunc for:", &self.pre_token);
            return Ok(None);
        }

        let mut lhs = self.parse_prefix_expr()?.unwrap();

        dbg!(&self.pre_token, &self.peek_token, &lhs);
        while self.pre_token != Token::Semicolon && precedence < self.peek_token_predence() {
            if !is_infix_oper(&self.peek_token) {
                dbg!(
                    "No infixOperatorFunc for {} lhs: {:?}",
                    &self.pre_token,
                    &lhs
                );
                return Ok(Some(lhs));
            }
            self.next_token();
            lhs = self.parse_infix_expr(lhs)?;
        }

        Ok(Some(lhs))
    }

    fn parse_prefix_expr(&mut self) -> Result<Option<Expression>> {
        // 1 + 2 + 3
        match self.pre_token.clone() {
            Token::Exclamation => {
                self.next_token();

                Ok(Some(Expression::Operation(Operation::Not(Box::new(
                    match self.parse_expression(Precedence::Prefix)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!("Operation::Not exp is None")));
                        }
                    },
                )))))
            }
            Token::Add => Ok(Some(Expression::Operation(Operation::Assert(Box::new(
                match self.parse_expression(Precedence::Prefix)? {
                    Some(exp) => exp,
                    None => {
                        return Err(Error::ParseErr(fmt_err!("Operation::Assert exp is None")));
                    }
                },
            ))))),
            Token::Minus => {
                self.next_token();
                Ok(Some(Expression::Operation(Operation::Negate(Box::new(
                    match self.parse_expression(Precedence::Prefix)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!("Operation::Negate exp is None")));
                        }
                    },
                )))))
            }
            Token::Number(n) => {
                // 如果包含 '.' 则说明是一个浮点数
                if n.contains('.') {
                    Ok(Some(Expression::Literal(Literal::Float(
                        match n.parse::<f64>() {
                            Ok(n) => n,
                            Err(e) => {
                                return Err(Error::ParseErr(fmt_err!("{}", e)));
                            }
                        },
                    ))))
                } else {
                    Ok(Some(Expression::Literal(Literal::Int(
                        match n.parse::<i64>() {
                            Ok(n) => n,
                            Err(e) => {
                                return Err(Error::ParseErr(fmt_err!("{}", e)));
                            }
                        },
                    ))))
                }
            }
            Token::LeftParen => {
                self.next_token();
                let exp = self.parse_expression(Precedence::Lowest);
                if !self.peek_if_token(Token::RightParen) {
                    return Ok(None);
                }

                exp
            }
            Token::Ident(i) => match &self.peek_token {
                Token::Period => {
                    self.next_token();
                    Ok(Some(Expression::Field(
                        Some(i),
                        match self.peek_token.clone() {
                            Token::Ident(i) => {
                                self.next_token();
                                i
                            }
                            _ => {
                                return Err(Error::ParseErr(fmt_err!("expected: Token::Ident")));
                            }
                        },
                    )))
                }
                _ => Ok(Some(Expression::Literal(Literal::String(i)))),
            },
            Token::String(s) => Ok(Some(Expression::Literal(Literal::String(s)))),
            Token::KeyWord(k) => match k {
                Keyword::True => Ok(Some(Expression::Literal(Literal::Bool(true)))),
                Keyword::False => Ok(Some(Expression::Literal(Literal::Bool(false)))),
                Keyword::Null => Ok(Some(Expression::Literal(Literal::Null))),
                _ => Err(Error::ParseErr(fmt_err!(
                    "No prefixOperatorFunc for {}",
                    self.pre_token
                ))),
            },

            _ => Err(Error::ParseErr(fmt_err!(
                "No prefixOperatorFunc for {}",
                self.pre_token
            ))),
        }
    }

    fn parse_infix_expr(&mut self, exp: Expression) -> Result<Expression> {
        match self.pre_token {
            Token::Add => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();
                Ok(Expression::Operation(Operation::Add(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!("Operation::Add exp is None")));
                        }
                    }),
                )))
            }
            Token::Equal => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::Equal(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!("Operation::Equal exp is None")));
                        }
                    }),
                )))
            }
            Token::GreaterThan => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::GreaterThan(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!(
                                "Operation::GreaterThan exp is None"
                            )));
                        }
                    }),
                )))
            }
            Token::GreaterThanOrEqual => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::GreaterThanOrEqual(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!(
                                "Operation::GreaterThanOrEqual exp is None"
                            )));
                        }
                    }),
                )))
            }
            Token::LessThan => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::LessThan(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!(
                                "Operation::LessThan exp is None"
                            )));
                        }
                    }),
                )))
            }
            Token::LessThanOrEqual => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::LessThanOrEqual(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!(
                                "Operation::LessThanOrEqual exp is None"
                            )));
                        }
                    }),
                )))
            }
            Token::Minus => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::Subtract(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!("Operation::Minus exp is None")));
                        }
                    }),
                )))
            }
            Token::NotEqual => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::NotEqual(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!(
                                "Operation::NotEqual exp is None"
                            )));
                        }
                    }),
                )))
            }
            Token::KeyWord(Keyword::And) => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::And(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!("Operation::And exp is None")));
                        }
                    }),
                )))
            }
            Token::KeyWord(Keyword::Or) => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::Or(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!("Operation::Or exp is None")));
                        }
                    }),
                )))
            }
            Token::KeyWord(Keyword::Like) => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::Like(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!("Operation::Like exp is None")));
                        }
                    }),
                )))
            }
            Token::Percent => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();
                Ok(Expression::Operation(Operation::Modulo(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!(
                                "Operation::Percent exp is None"
                            )));
                        }
                    }),
                )))
            }
            Token::Asterisk => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();
                Ok(Expression::Operation(Operation::Multiply(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!(
                                "Operation::Asterisk exp is None"
                            )));
                        }
                    }),
                )))
            }
            Token::Slash => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();
                Ok(Expression::Operation(Operation::Divide(
                    Box::new(exp),
                    Box::new(match self.parse_expression(precedence)? {
                        Some(exp) => exp,
                        None => {
                            return Err(Error::ParseErr(fmt_err!("Operation::Slash exp is None")));
                        }
                    }),
                )))
            }
            // 如果 ( 是一个中缀运算符, 则是一个函数
            Token::LeftParen => Ok(Expression::Function(
                match exp {
                    Expression::Literal(Literal::String(s)) => s,
                    _ => {
                        return Err(Error::ParseErr(fmt_err!(
                            "Operation::LeftParen exp is not Literal::String"
                        )));
                    }
                },
                match self.parse_expression_list()? {
                    Some(exprs) => exprs,
                    None => {
                        return Err(Error::ParseErr(fmt_err!(
                            "Operation::LeftParen exp is None"
                        )));
                    }
                },
            )),
            _ => Err(Error::ParseErr(fmt_err!(
                "No infixOperatorFunc for {}",
                self.pre_token
            ))),
        }
    }

    // (1, 3, 4)
    fn parse_expression_list(&mut self) -> Result<Option<Vec<Expression>>> {
        let mut exprs = Vec::new();

        if self.peek_if_token(Token::RightParen) {
            self.next_token();
            return Ok(Some(exprs));
        }

        self.next_token();
        exprs.push(match self.parse_expression(Precedence::Lowest)? {
            Some(exp) => exp,
            None => {
                return Err(Error::ParseErr(fmt_err!(
                    "Operation::LeftParen exp is None"
                )));
            }
        });

        while self.peek_if_token(Token::Comma) {
            self.next_token();

            exprs.push(match self.parse_expression(Precedence::Lowest)? {
                Some(exp) => exp,
                None => {
                    return Err(Error::ParseErr(fmt_err!(
                        "parse_expression_list exp is None"
                    )));
                }
            });
        }

        if !self.peek_if_token(Token::RightParen) {
            return Ok(None);
        }

        Ok(Some(exprs))
    }

    fn peek_token_predence(&mut self) -> Precedence {
        operator::match_precedence(self.peek_token.clone())
    }
}

#[cfg(test)]
pub mod test {

    use std::vec;

    use super::stmt::*;
    use super::*;
    use log::{debug, error};
    use std::io::Write;

    #[cfg(test)]
    static LOG_INIT: std::sync::Once = std::sync::Once::new();

    #[test]
    pub fn init() {
        LOG_INIT.call_once(|| {
            env_logger::Builder::new()
                .format(|buf, record| {
                    writeln!(
                        buf,
                        "{} {} {}:{} {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                        record.level(),
                        record.file().unwrap(),
                        record.line().unwrap(),
                        record.args()
                    )
                })
                .filter(None, log::LevelFilter::Debug)
                .init();
        });
    }

    #[test]
    fn parse_create_test() {
        let sql = "create table shaun (c1 int, c2 string, c3 text);";
        let mut parser = Parser::new_parser(sql.to_owned());
        let result = Statement::CreateTable(stmt::CreateTableStmt {
            columns: vec![
                column::Column {
                    name: "c1".to_string(),
                    data_type: DataType::Int,
                    primary_key: false,
                    nullable: None,
                    default: None,
                    unique: false,
                    index: false,
                    references: None,
                },
                column::Column {
                    name: "c2".to_string(),
                    data_type: DataType::String,
                    primary_key: false,
                    nullable: None,
                    default: None,
                    unique: false,
                    index: false,
                    references: None,
                },
                column::Column {
                    name: "c3".to_string(),
                    data_type: DataType::String,
                    primary_key: false,
                    nullable: None,
                    default: None,
                    unique: false,
                    index: false,
                    references: None,
                },
            ],
            table_name: "shaun".to_string(),
        });
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(err) => {
                error!("get error: {}", err);
                assert!(false)
            }
        }
    }

    #[test]
    fn parse_transaction_test() {
        init();

        let mut sql = "begin transaction;";
        let mut parser = Parser::new_parser(sql.to_owned());
        let mut result = Statement::Begin(BeginStmt {
            is_readonly: false,
            version: None,
        });
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(err) => {
                error!("expected: {} but get {}", result, err);
                assert!(false);
            }
        }

        sql = "begin transaction read only;";
        result = Statement::Begin(BeginStmt {
            is_readonly: true,
            version: None,
        });

        parser.update(sql);
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(err) => {
                error!("expected: {} but get {}", result, err);
                assert!(false);
            }
        }

        sql = "begin;";
        result = Statement::Begin(BeginStmt {
            is_readonly: false,
            version: None,
        });

        parser.update(sql);
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {} but get {}", result, e);
                assert!(false);
            }
        }

        sql = "commit;";
        result = Statement::Commit;

        parser.update(sql);
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {} but get {}", result, e);
                assert!(false);
            }
        }

        sql = "rollback";
        result = Statement::Rollback;
        parser.update(sql);
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {} but get {}", result, e);
                assert!(false);
            }
        }

        sql = "BEGIN TRANSACTION READ WRITE";
        result = Statement::Begin(BeginStmt {
            is_readonly: false,
            version: None,
        });
        parser.update(sql);
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {} but get {}", result, e);
                assert!(false);
            }
        }

        sql = "BEGIN TRANSACTION READ ONLY AS OF SYSTEM TIME 129012313;";
        result = Statement::Begin(BeginStmt {
            is_readonly: true,
            version: Some(129012313),
        });
        parser.update(sql);
        match parser.parse_stmt() {
            Ok(s) => {
                assert_eq!(result, s);
            }
            Err(e) => {
                error!("expected: {} but get {}", result, e);
                assert!(false);
            }
        }
    }

    #[test]
    fn parse_select_test() {
        let p = Parser::new_parser("SELECT c1 AS c2 FROM table_1".to_owned());
    }
}
