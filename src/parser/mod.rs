mod column;
mod data_type;
mod expression;
mod keyword;
pub mod lexer;
mod operation;
mod operator;
mod stmt;
pub mod token;

use crate::parser::{operation::Operation, operator::match_precedence};

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

    pub fn parse_stmt(&mut self) -> Result<Statement, String> {
        // 直接与 lexer 产生的第一个 Token 作比较
        if self.pre_token == Token::Eof {
            return Err(format!("empty token {}", self.pre_token));
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

            t => Err(format!("unexpected token: {}", t)),
        };

        result
    }

    fn parse_transaction_stmt(&mut self) -> Result<Statement, String> {
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

                        t => return Err(format!("unexpected token: {}", t)),
                    }
                }

                if self.next_if_keyword(Keyword::As) {
                    self.next_expected_keyword(Keyword::Of)?;
                    self.next_expected_keyword(Keyword::System)?;
                    self.next_expected_keyword(Keyword::Time)?;

                    match self.next_token() {
                        Token::Number(n) => version = n.parse::<u64>().ok(),
                        t => {
                            return Err(format!("unexpected token: {} expected: Number", t));
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

            t => Err(format!("unexpected token: {}", t)),
        }
    }

    #[allow(dead_code)]
    fn parse_ddl_stmt(&mut self) -> Result<Statement, String> {
        unimplemented!()
    }

    fn parse_delete_stmt(&mut self) -> Result<Statement, String> {
        unimplemented!()
    }

    fn parse_insert_stmt(&mut self) -> Result<Statement, String> {
        unimplemented!()
    }

    fn parse_create_stmt(&mut self) -> Result<Statement, String> {
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
                    return Err(format!(
                        "unexpected token {:?}, want Comma or RightParen",
                        token
                    ));
                }
            }
        }

        self.next_expected_token(Token::Semicolon)?;

        Ok(Statement::CreateTable(stmt::CreateTableStmt {
            columns,
            table_name,
        }))
    }

    fn parse_column(&mut self) -> Result<Column, String> {
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
                    return Err(format!("unexpected token: {}", t));
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

    fn parse_drop_stmt(&mut self) -> Result<Statement, String> {
        unimplemented!()
    }

    fn parse_select_stmt(&mut self) -> Result<Statement, String> {
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
            limit: if self.next_if_keyword(Keyword::Limit) {
                Some(self.parse_expression(Precedence::Lowest)?.unwrap())
            } else {
                None
            },
            offset: if self.next_if_keyword(Keyword::Offset) {
                Some(self.parse_expression(Precedence::Lowest)?.unwrap())
            } else {
                None
            },
        }))
    }

    fn parse_clause_select(&mut self) -> Result<Vec<(Expression, Option<String>)>, String> {
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
            debug!(
                "pre token: {:?} peek_token: {:?}",
                &self.pre_token, &self.peek_token
            );
            self.next_token();

            let alias = match self.pre_token.clone() {
                Token::KeyWord(Keyword::As) => {
                    self.next_token();
                    match self.pre_token.clone() {
                        Token::Ident(ident) => Some(ident),
                        _ => None,
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

    fn parse_clause_from(&mut self) -> Result<Vec<FromItem>, String> {
        let mut froms = Vec::new();

        if !self.next_if_keyword(Keyword::From) {
            return Ok(froms);
        }

        loop {
            let mut item = self.parse_clause_from_table()?;

            loop {
                let join_type = self.parse_clause_from_jointype()?;
                if join_type.is_none() {
                    break;
                }
                let join_type = join_type.unwrap();

                let left_exp = Box::new(item);
                let right_exp = Box::new(self.parse_clause_from_table()?);
                let predicate = match join_type {
                    JoinType::Outer => None,
                    _ => {
                        self.next_expected_keyword(Keyword::On)?;

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

    fn parse_clause_from_table(&mut self) -> Result<FromItem, String> {
        let name = self.next_ident()?;

        self.next_token();

        let alias = match self.pre_token.clone() {
            Token::KeyWord(Keyword::As) => {
                self.next_token();
                match self.pre_token.clone() {
                    Token::Ident(ident) => Some(ident),
                    _ => None,
                }
            }
            Token::Ident(ident) => Some(ident),
            _ => None,
        };
        Ok(FromItem::Table { name, alias })
    }

    fn parse_clause_from_jointype(&mut self) -> Result<Option<JoinType>, String> {
        if self.next_if_token(Token::KeyWord(Keyword::Outer)) {
            self.next_expected_keyword(Keyword::Join)?;

            Ok(Some(JoinType::Outer))
        } else if self.next_if_token(Token::KeyWord(Keyword::Inner)) {
            self.next_expected_keyword(Keyword::Join)?;

            Ok(Some(JoinType::Inner))
        } else if self.next_if_token(Token::KeyWord(Keyword::Left)) {
            self.next_expected_keyword(Keyword::Outer)?;
            self.next_expected_keyword(Keyword::Join)?;

            Ok(Some(JoinType::Left))
        } else if self.next_if_token(Token::KeyWord(Keyword::Right)) {
            self.next_expected_keyword(Keyword::Outer)?;
            self.next_expected_keyword(Keyword::Join)?;

            Ok(Some(JoinType::Right))
        } else {
            Ok(None)
        }
    }

    fn parse_clause_group_by(&mut self) -> Result<Vec<Expression>, String> {
        let mut exprs = Vec::new();

        if self.next_if_keyword(Keyword::Group) {
            return Ok(exprs);
        }

        self.next_expected_keyword(Keyword::By)?;

        loop {
            exprs.push(self.parse_expression(Precedence::Lowest)?.unwrap());

            if !self.next_if_token(Token::Comma) {
                break;
            }
        }

        Ok(exprs)
    }

    fn parse_clause_where(&mut self) -> Result<Option<Expression>, String> {
        if !self.next_if_keyword(Keyword::Where) {
            return Ok(None);
        }

        return Ok(Some(self.parse_expression(Precedence::Lowest)?.unwrap()));
    }

    fn parse_clause_having(&mut self) -> Result<Option<Expression>, String> {
        if self.next_if_keyword(Keyword::Having) {
            Ok(Some(self.parse_expression(Precedence::Lowest)?.unwrap()))
        } else {
            Ok(None)
        }
    }

    fn parse_clause_order(&mut self) -> Result<Vec<(Expression, OrderByType)>, String> {
        if !self.next_if_keyword(Keyword::Order) {
            return Ok(Vec::new());
        }

        self.next_expected_keyword(Keyword::By)?;
        let mut orders = Vec::new();

        loop {
            orders.push((
                self.parse_expression(Precedence::Lowest)?.unwrap(),
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

    fn parse_update_stmt(&mut self) -> Result<Statement, String> {
        unimplemented!()
    }
    fn parse_explain_stmt(&mut self) -> Result<Statement, String> {
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

    fn next_expected_keyword(&mut self, k: Keyword) -> Result<(), String> {
        let t = self.next_token();

        if *t == Token::KeyWord(k) {
            Ok(())
        } else {
            Err(format!("unexpected keyword: {} want: {}", t, k))
        }
    }

    fn next_expected_token(&mut self, t: Token) -> Result<(), String> {
        let token = self.next_token();

        if *token == t {
            Ok(())
        } else {
            Err(format!("unexpected token: {} want: {}", token, t))
        }
    }

    fn next_ident(&mut self) -> Result<String, String> {
        match self.next_token() {
            Token::Ident(ident) => Ok(ident.clone()),
            t => Err(format!("expected: Token::Ident but get: {}", t)),
        }
    }

    // (1 + 2)
    fn parse_expression(&mut self, precedence: Precedence) -> Result<Option<Expression>, String> {
        if !self.is_prefix_oper() {
            debug!("No prefixOperatorFunc for: {}", &self.pre_token);
            return Ok(None);
        }

        let mut lhs = self.parse_prefix_expr()?.unwrap();

        debug!(
            "pre_token {:?} peek_token: {:?} lhs: {:?}",
            self.pre_token, self.peek_token, lhs
        );
        debug!("{:?} {:?}", precedence, self.peek_token_predence());
        while self.pre_token != Token::Semicolon && precedence < self.peek_token_predence() {
            if !self.is_infix_oper() {
                debug!(
                    "No infixOperatorFunc for {} lhs: {:?}",
                    &self.pre_token, lhs
                );
                return Ok(Some(lhs));
            }
            self.next_token();
            lhs = self.parse_infix_expr(lhs)?;
        }

        Ok(Some(lhs))
    }

    fn parse_prefix_expr(&mut self) -> Result<Option<Expression>, String> {
        // 1 + 2 + 3
        match self.pre_token.clone() {
            Token::Exclamation => {
                self.next_token();

                Ok(Some(Expression::Operation(Operation::Not(Box::new(
                    self.parse_expression(Precedence::Prefix)?.unwrap(),
                )))))
            }
            Token::Add => Ok(Some(Expression::Operation(Operation::Assert(Box::new(
                self.parse_expression(Precedence::Prefix)?.unwrap(),
            ))))),
            Token::Minus => {
                self.next_token();
                Ok(Some(Expression::Operation(Operation::Negate(Box::new(
                    self.parse_expression(Precedence::Prefix)?.unwrap(),
                )))))
            }
            Token::Number(n) => {
                // 如果包含 '.' 则说明是一个浮点数
                if n.contains('.') {
                    Ok(Some(Expression::Literal(Literal::Float(
                        match n.parse::<f64>() {
                            Ok(n) => n,
                            Err(e) => {
                                return Err(format!("ParseErr: {}", e));
                            }
                        },
                    ))))
                } else {
                    Ok(Some(Expression::Literal(Literal::Int(
                        match n.parse::<i64>() {
                            Ok(n) => n,
                            Err(e) => {
                                return Err(format!("ParseErr: {}", e));
                            }
                        },
                    ))))
                }
            }
            Token::LeftParen => {
                self.next_token();
                let exp = self.parse_expression(Precedence::Lowest);
                debug!("{:?} {:?}", self.pre_token, self.peek_token);
                if !self.peek_if_token(Token::RightParen) {
                    return Ok(None);
                }

                exp
            }
            _ => Err(format!("No prefixOperatorFunc for {}", self.pre_token)),
        }
    }

    fn parse_infix_expr(&mut self, exp: Expression) -> Result<Expression, String> {
        match self.pre_token {
            Token::Add => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();
                Ok(Expression::Operation(Operation::Add(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::Equal => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::Equal(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::GreaterThan => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::GreaterThan(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::GreaterThanOrEqual => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::GreaterThanOrEqual(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::LessThan => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::LessThan(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::LessThanOrEqual => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::LessThanOrEqual(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::Minus => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::Subtract(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::NotEqual => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::NotEqual(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::KeyWord(Keyword::And) => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::And(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::KeyWord(Keyword::Or) => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::Or(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::KeyWord(Keyword::Like) => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();

                Ok(Expression::Operation(Operation::Like(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::Percent => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();
                Ok(Expression::Operation(Operation::Modulo(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::Asterisk => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();
                Ok(Expression::Operation(Operation::Multiply(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            Token::Slash => {
                let precedence = match_precedence(self.pre_token.clone());
                self.next_token();
                Ok(Expression::Operation(Operation::Divide(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence)?.unwrap()),
                )))
            }
            _ => Err(format!("No infixOperatorFunc for {}", self.pre_token)),
        }
    }

    fn is_prefix_oper(&self) -> bool {
        match self.pre_token {
            Token::Exclamation => true,
            Token::Minus => true,
            Token::Add => true,
            Token::Number(_) => true,
            Token::LeftParen => true,
            _ => false,
        }
    }

    fn is_infix_oper(&self) -> bool {
        match self.peek_token {
            Token::Add => true,
            Token::Equal => true,
            Token::GreaterThan => true,
            Token::GreaterThanOrEqual => true,
            Token::LessThan => true,
            Token::LessThanOrEqual => true,
            Token::Minus => true,
            Token::NotEqual => true,
            Token::Percent => true,
            Token::Slash => true,
            Token::Asterisk => true,
            Token::Caret => true,
            Token::KeyWord(keyword::Keyword::And) => true,
            Token::KeyWord(Keyword::Like) => true,
            Token::KeyWord(Keyword::Or) => true,
            _ => false,
        }
    }

    fn peek_token_predence(&mut self) -> Precedence {
        operator::match_precedence(self.peek_token.clone())
    }
}

#[cfg(test)]
mod test {

    use std::vec;

    use super::stmt::*;
    use super::*;
    use log::{debug, error};
    use std::io::Write;

    #[cfg(test)]
    static LOG_INIT: std::sync::Once = std::sync::Once::new();

    #[test]
    fn init() {
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
    fn parse_expression_test() {
        init();

        // Token::Number("123") Token::Plus Token::Number("456");
        let mut parser = Parser::new_parser("SELECT 123 + 456;".to_owned());
        let mut result_exp = Expression::Operation(Operation::Add(
            Box::new(Expression::Literal(Literal::Int(123))),
            Box::new(Expression::Literal(Literal::Int(456))),
        ));
        let mut expr_selects = vec![];
        expr_selects.push((result_exp.clone(), None));
        let mut result = Statement::Select(SelectStmt {
            selects: expr_selects,
            froms: vec![],
            wheres: None,
            group_by: None,
            having: None,
            order: vec![],
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
        let mut result = Statement::Select(SelectStmt {
            selects: expr_selects,
            froms: vec![],
            wheres: None,
            group_by: None,
            having: None,
            order: vec![],
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

        debug!("test 2");
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
        let mut result = Statement::Select(SelectStmt {
            selects: expr_selects.clone(),
            froms: vec![],
            wheres: None,
            group_by: None,
            having: None,
            order: vec![],
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
        let mut result = Statement::Select(SelectStmt {
            selects: expr_selects.clone(),
            froms: vec![],
            wheres: None,
            group_by: None,
            having: None,
            order: vec![],
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
        let mut result = Statement::Select(SelectStmt {
            selects: expr_selects.clone(),
            froms: vec![],
            wheres: None,
            group_by: None,
            having: None,
            order: vec![],
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
        let mut result = Statement::Select(SelectStmt {
            selects: expr_selects.clone(),
            froms: vec![],
            wheres: None,
            group_by: None,
            having: None,
            order: vec![],
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
    }

    #[test]
    fn parse_select_test() {
        let p = Parser::new_parser("SELECT c1 AS c2 FROM table_1".to_owned());

    }
}
