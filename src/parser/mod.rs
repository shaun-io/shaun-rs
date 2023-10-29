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
use crate::parser::stmt::{AlterStmt, AlterType, CreateIndexStmt, DeleteTableStmt};
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
use std::collections::BTreeMap;
use stmt::Statement;
use token::Token;

use self::stmt::{
    DropTableStmt, ExplainStmt, SetStmt, SetVariableType, TransactionIsolationLevel, UpdateStmt,
};
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

    pub fn update(&mut self, sql_str: &str) -> &mut Self {
        self.lexer.update(sql_str.to_owned());
        self.peek_token = self.lexer.next_token();
        self.pre_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();

        self
    }

    pub fn parse_stmt(&mut self) -> Result<Statement> {
        // 直接与 lexer 产生的第一个 Token 作比较
        if self.pre_token == Token::Eof {
            return Err(Error::Parse(fmt_err!("empty token {}", self.pre_token)));
        }

        match &self.pre_token {
            Token::KeyWord(Keyword::Begin)
            | Token::KeyWord(Keyword::Commit)
            | Token::KeyWord(Keyword::Rollback) => self.parse_transaction_stmt(),

            Token::KeyWord(Keyword::Create) => self.parse_create_stmt(),
            Token::KeyWord(Keyword::Drop) => self.parse_drop_stmt(),

            Token::KeyWord(Keyword::Delete) => self.parse_delete_stmt(),
            Token::KeyWord(Keyword::Insert) => self.parse_insert_stmt(),
            Token::KeyWord(Keyword::Select) => self.parse_select_stmt(),
            Token::KeyWord(Keyword::Update) => self.parse_update_stmt(),
            Token::KeyWord(Keyword::Alter) => self.parse_alter_stmt(),

            Token::KeyWord(Keyword::Show) => self.parse_show_stmt(),

            Token::KeyWord(Keyword::Explain) => self.parse_explain_stmt(),
            Token::KeyWord(Keyword::Describe) => match &self.peek_token {
                Token::Ident(i) => Ok(Statement::DescribeTable(i.to_owned())),
                _ => Err(Error::Parse(fmt_err!(
                    "unexpected token: {}",
                    &self.peek_token
                ))),
            },
            Token::KeyWord(Keyword::Set) => self.parse_set_stmt(),

            t => Err(Error::Parse(fmt_err!("unexpected token: {}", t))),
        }
    }

    fn parse_set_stmt(&mut self) -> Result<Statement> {
        let mut is_session = true;
        match &self.peek_token {
            Token::KeyWord(Keyword::Session) => {
                if !is_session {
                    return Err(Error::Parse(fmt_err!("SET SESSION GLOBAL is not valid!")));
                }
                self.next_token();
                match self.peek_token.clone() {
                    Token::KeyWord(Keyword::Transaction) => Ok(Statement::Set(SetStmt {
                        set_value: self.parse_set_transaction()?,

                        is_session,
                    })),
                    t => Err(Error::Parse(fmt_err!("unexpected token: {}", t))),
                }
            }
            Token::KeyWord(Keyword::Global) => {
                is_session = false;

                self.next_token();
                match self.peek_token.clone() {
                    Token::KeyWord(Keyword::Transaction) => Ok(Statement::Set(SetStmt {
                        set_value: self.parse_set_transaction()?,
                        is_session,
                    })),
                    t => Err(Error::Parse(fmt_err!("unexpected token: {}", t))),
                }
            }
            Token::KeyWord(Keyword::Transaction) => Ok(Statement::Set(SetStmt {
                set_value: self.parse_set_transaction()?,
                is_session,
            })),
            t => {
                // TODO: 需要支持 SET @var_name = expression;
                Err(Error::Parse(fmt_err!("unexpected token: {}", t)))
            }
        }
    }

    fn parse_set_transaction(&mut self) -> Result<SetVariableType> {
        self.next_token();
        self.next_expected_keyword(Keyword::Isolation)?;
        self.next_expected_keyword(Keyword::Level)?;

        match &self.peek_token {
            Token::KeyWord(Keyword::Read) => {
                self.next_token();
                match self.peek_token.clone() {
                    Token::KeyWord(Keyword::Committed) => Ok(SetVariableType::Transaction(
                        TransactionIsolationLevel::ReadCommitted,
                    )),

                    Token::KeyWord(Keyword::Uncommitted) => Ok(SetVariableType::Transaction(
                        TransactionIsolationLevel::ReadUncommitted,
                    )),

                    t => Err(Error::Parse(fmt_err!("unexpected token: {}", t))),
                }
            }
            Token::KeyWord(Keyword::Repeatable) => {
                self.next_token();
                match self.peek_token.clone() {
                    Token::KeyWord(Keyword::Read) => Ok(SetVariableType::Transaction(
                        TransactionIsolationLevel::RepeatableRead,
                    )),

                    t => Err(Error::Parse(fmt_err!("unexpected token: {}", t))),
                }
            }
            Token::KeyWord(Keyword::Serializable) => Ok(SetVariableType::Transaction(
                TransactionIsolationLevel::Serializable,
            )),
            t => Err(Error::Parse(fmt_err!("unexpected token: {}", t))),
        }
    }

    fn parse_show_stmt(&mut self) -> Result<Statement> {
        match &self.peek_token {
            Token::KeyWord(Keyword::Tables) => Ok(Statement::ShowTables),
            Token::KeyWord(Keyword::Databases) => Ok(Statement::ShowDatabase),
            _ => Err(Error::Parse(fmt_err!(
                "unexpected token: {}",
                &self.peek_token
            ))),
        }
    }

    fn parse_alter_stmt(&mut self) -> Result<Statement> {
        // ALTER TABLE table_name
        self.next_expected_keyword(Keyword::Table)?;

        let table_name = self.next_ident()?;
        self.next_token();

        match &self.pre_token {
            Token::KeyWord(Keyword::Add) => match &self.peek_token {
                Token::KeyWord(Keyword::Column) => {
                    // ALTER TABLE table_name ADD COLUMN new_column_name column_data_type
                    self.next_token();
                    dbg!(&self.pre_token);
                    let new_column = self.parse_column()?;

                    Ok(Statement::Alter(AlterStmt {
                        alter_type: AlterType::AddColumn(new_column),
                        table_name,
                    }))
                }
                Token::Ident(_) => {
                    // ALTER TABLE table_name ADD new_column_name column_data_type;
                    self.next_token();
                    let new_column = self.parse_column()?;

                    Ok(Statement::Alter(AlterStmt {
                        alter_type: AlterType::AddColumn(new_column),
                        table_name,
                    }))
                }

                Token::KeyWord(Keyword::Index) => {
                    // ALTER TABLE table_name ADD INDEX index_name(option) (column_1_name, xxx);
                    self.next_token();

                    let add_index_name = match self.peek_token.clone() {
                        Token::Ident(i) => {
                            self.next_token();
                            Some(i)
                        }
                        _ => None,
                    };
                    let mut column_list = vec![];
                    match self.peek_token.clone() {
                        Token::LeftParen => {
                            self.next_token();
                            loop {
                                match self.peek_token.clone() {
                                    Token::Ident(i) => {
                                        self.next_token();
                                        column_list.push(i);
                                    }
                                    Token::Comma => {
                                        self.next_token();
                                        continue;
                                    }
                                    Token::RightParen => {
                                        self.next_token();
                                        break;
                                    }
                                    _ => {
                                        return Err(Error::Parse(fmt_err!(
                                            "expected Token::LeftParen, but get: {}",
                                            &self.peek_token
                                        )));
                                    }
                                }
                            }

                            Ok(Statement::Alter(AlterStmt {
                                alter_type: AlterType::AddIndex(add_index_name, column_list),
                                table_name,
                            }))
                        }
                        _ => Err(Error::Parse(fmt_err!(
                            "expected Token::LeftParen, but get: {}",
                            &self.peek_token
                        ))),
                    }
                }
                _ => Err(Error::Parse(fmt_err!(
                    "unexpected token: {}",
                    &self.peek_token
                ))),
            },
            Token::KeyWord(Keyword::Drop) => {
                match &self.peek_token {
                    Token::KeyWord(Keyword::Column) => {
                        // ALTER TABLE table_name
                        // DROP column_name;
                        self.next_token();
                        let column_name = self.next_ident()?;
                        Ok(Statement::Alter(AlterStmt {
                            alter_type: AlterType::DropColumn(column_name),
                            table_name,
                        }))
                    }
                    Token::KeyWord(Keyword::Index) => {
                        // ALTER TABLE table_name
                        // DROP INDEX index_name;
                        self.next_token();
                        let index_name = self.next_ident()?;

                        Ok(Statement::Alter(AlterStmt {
                            alter_type: AlterType::RemoveIndex(index_name),
                            table_name,
                        }))
                    }
                    _ => Err(Error::Parse(fmt_err!(
                        "unexpected token: {}",
                        &self.peek_token
                    ))),
                }
            }
            Token::KeyWord(Keyword::Rename) => {
                match &self.peek_token {
                    Token::KeyWord(Keyword::Column) => {
                        // ALTER TABLE table_name RENAME COLUMN
                        // old_column_name TO new_column_name;
                        self.next_token();
                        let old_column_name = self.next_ident()?;
                        self.next_expected_keyword(Keyword::To)?;
                        let new_column_name = self.next_ident()?;

                        Ok(Statement::Alter(AlterStmt {
                            alter_type: AlterType::RenameColumn(old_column_name, new_column_name),
                            table_name,
                        }))
                    }
                    Token::KeyWord(Keyword::To) => {
                        // ALTER TABLE table_name RENAME TO new_table_name;
                        self.next_token();
                        let new_table_name = self.next_ident()?;

                        Ok(Statement::Alter(AlterStmt {
                            alter_type: AlterType::RenameTable(new_table_name),
                            table_name,
                        }))
                    }
                    _ => Err(Error::Parse(fmt_err!(
                        "unexpected token: {}",
                        &self.peek_token
                    ))),
                }
            }
            Token::KeyWord(Keyword::Modify) => {
                // 修改列的属性
                // ALTER TABLE table_name MODIFY
                // column_name column_data_type;
                self.next_token();
                Ok(Statement::Alter(AlterStmt {
                    alter_type: AlterType::ModifyColumn(self.parse_column()?),
                    table_name,
                }))
            }
            _ => Err(Error::Parse(fmt_err!(
                "ALTER TABLE is not valid unexpected token: {}",
                &self.pre_token
            ))),
        }
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

                        t => return Err(Error::Parse(fmt_err!("unexpected token: {}", t))),
                    }
                }

                if self.next_if_keyword(Keyword::As) {
                    self.next_expected_keyword(Keyword::Of)?;
                    self.next_expected_keyword(Keyword::System)?;
                    self.next_expected_keyword(Keyword::Time)?;

                    match self.next_token() {
                        Token::Number(n) => version = n.parse::<u64>().ok(),
                        t => {
                            return Err(Error::Parse(fmt_err!(
                                "unexpected token: {} expected: Number",
                                t
                            )));
                        }
                    }
                }

                Ok(Statement::Begin(stmt::BeginStmt {
                    is_readonly,
                    version,
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

            t => Err(Error::Parse(fmt_err!("unexpected token: {}", t))),
        }
    }

    fn parse_delete_stmt(&mut self) -> Result<Statement> {
        self.next_expected_keyword(Keyword::From)?;
        let table_name = self.next_ident()?;
        self.next_token();
        Ok(Statement::Delete(DeleteTableStmt {
            table_name,
            r#where: self.parse_clause_where()?,
        }))
    }

    fn parse_insert_stmt(&mut self) -> Result<Statement> {
        self.next_expected_keyword(Keyword::Into)?;
        let table_name = self.next_ident()?;
        let columns = if self.peek_token == Token::LeftParen {
            // todo: 这里应该有一个 next_if_peek 的方法处理这个情况
            self.next_token();
            let mut cols = Vec::new();
            loop {
                cols.push(self.next_ident()?);
                match self.next_token() {
                    Token::Comma => continue,
                    Token::RightParen => break,
                    t => {
                        return Err(Error::Parse(fmt_err!(
                            "excepted Comma or RightParen, get {}",
                            t
                        )));
                    }
                }
            }
            Some(cols)
        } else {
            None
        };

        self.next_expected_keyword(Keyword::Values)?;
        let mut values = Vec::new();

        loop {
            if self.peek_token != Token::LeftParen {
                return Err(Error::Parse(fmt_err!(
                    "except token LeftParen, get {:?}",
                    self.peek_token
                )));
            }

            let mut exprs = Vec::new();
            self.next_token();

            loop {
                self.next_token();
                exprs.push(Some(self.parse_expression(Precedence::Lowest)?));
                match self.next_token() {
                    Token::RightParen => break,
                    Token::Comma => {}
                    _ => return Err(Error::Parse("".to_owned())),
                }
            }

            values.push(exprs);
            if self.peek_token != Token::Comma {
                break;
            }
        }

        Ok(Statement::Insert(stmt::InsertStmt {
            table_name,
            columns,
            values,
        }))
    }

    fn parse_create_index_stmt(&mut self) -> Result<Statement> {
        // CREATE [UNIQUE] INDEX [index_name]
        // ON [table_name] (column_name_1, column_name_2);
        let is_unique = match self.peek_token {
            Token::KeyWord(Keyword::Unique) => {
                self.next_token();
                match self.peek_token.clone() {
                    Token::KeyWord(Keyword::Index) => true,
                    _ => {
                        return Err(Error::Parse(fmt_err!(
                            "unexpected token: {}",
                            self.peek_token
                        )));
                    }
                }
            }
            _ => false,
        };
        self.next_token();
        let index_name = self.next_ident()?;
        self.next_expected_keyword(Keyword::On)?;
        let table_name = self.next_ident()?;
        match self.peek_token {
            Token::LeftParen => {}
            _ => {
                return Err(Error::Parse(fmt_err!(
                    "unexpected token: {}",
                    self.peek_token
                )));
            }
        };
        self.next_token();
        let mut columns = Vec::new();
        loop {
            columns.push(self.next_ident()?);
            let token = self.next_token();

            match token {
                Token::Comma => continue,
                Token::RightParen => break,
                _ => {
                    return Err(Error::Parse(fmt_err!(
                        "unexpected token {:?}, want Comma or RightParen",
                        token
                    )));
                }
            }
        }

        Ok(Statement::CreateIndex(CreateIndexStmt {
            is_unique,
            index_name,
            table_name,
            columns,
        }))
    }

    fn parse_create_stmt(&mut self) -> Result<Statement> {
        match self.peek_token {
            Token::KeyWord(Keyword::Table) => self.parse_create_table_stmt(),
            Token::KeyWord(Keyword::Unique) | Token::KeyWord(Keyword::Index) => {
                self.parse_create_index_stmt()
            }
            _ => Err(Error::Parse(fmt_err!(
                "unexpected token: {}",
                self.peek_token
            ))),
        }
    }

    fn parse_create_table_stmt(&mut self) -> Result<Statement> {
        // CREATE TABLE table_name
        //  (xxx_name xxx_addr xxx_addr xxx_addr,
        //  xxx, xxx);;
        self.next_expected_keyword(Keyword::Table)?;
        let name = self.next_ident();
        let table_name = match name {
            Ok(n) => n,
            Err(e) => {
                return Err(e);
            }
        };
        self.next_expected_token(Token::LeftParen)?;

        let mut columns = Vec::new();
        loop {
            columns.push(self.parse_column()?);
            let token = self.next_token();

            match token {
                Token::Comma => continue,
                Token::RightParen => break,
                _ => {
                    return Err(Error::Parse(fmt_err!(
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
        let column_name = self.next_ident()?;

        let mut column = column::Column {
            name: column_name,
            data_type: match self.next_token() {
                Token::KeyWord(Keyword::Bool) => DataType::Bool,
                Token::KeyWord(Keyword::Boolean) => DataType::Bool,

                Token::KeyWord(Keyword::Float) => DataType::Float32,
                Token::KeyWord(Keyword::Double) => DataType::Float64,

                Token::KeyWord(Keyword::Int) => DataType::Int32,
                Token::KeyWord(Keyword::Integer) => DataType::Int32,
                Token::KeyWord(Keyword::Int8) => DataType::Int8,
                Token::KeyWord(Keyword::Int16) => DataType::Int16,
                Token::KeyWord(Keyword::Int32) => DataType::Int32,
                Token::KeyWord(Keyword::Int64) => DataType::Int64,
                Token::KeyWord(Keyword::Uint8) => DataType::Uint8,
                Token::KeyWord(Keyword::Uint16) => DataType::Uint16,
                Token::KeyWord(Keyword::Uint32) => DataType::Uint32,
                Token::KeyWord(Keyword::Uint64) => DataType::Uint64,
                Token::KeyWord(Keyword::Float32) => DataType::Float32,
                Token::KeyWord(Keyword::Float64) => DataType::Float64,

                Token::KeyWord(Keyword::Text) => DataType::String,
                Token::KeyWord(Keyword::VarChar) => {
                    self.next_expected_token(Token::LeftParen)?;
                    let token = self.next_token();
                    let len = match token {
                        Token::Number(n) => match n.parse::<usize>() {
                            Ok(l) => l,
                            Err(e) => {
                                return Err(Error::Parse(fmt_err!("parse err: {}", e)));
                            }
                        },
                        t => {
                            return Err(Error::Parse(fmt_err!(
                                "unexpected token: {} expected: Number",
                                t
                            )));
                        }
                    };

                    self.next_expected_token(Token::RightParen)?;
                    DataType::Varchar(len)
                }
                Token::KeyWord(Keyword::Char) => DataType::Char,
                Token::KeyWord(Keyword::String) => DataType::String,

                t => {
                    return Err(Error::Parse(fmt_err!("unexpected token: {}", t)));
                }
            },
            primary_key: false,
            nullable: None,
            default: None,
            unique: false,
            index: false,
            references: None,
        };

        while let Token::KeyWord(keyword) = self.peek_token {
            match keyword {
                Keyword::Primary => {
                    self.next_token();
                    self.next_expected_keyword(Keyword::Key)?;
                    column.primary_key = true;
                }
                Keyword::Null => {
                    self.next_token();
                    if let Some(false) = column.nullable {
                        return Err(Error::Parse(fmt_err!(
                            "Column {} can't be both not nullable and nullable",
                            column.name
                        )));
                    }
                    column.nullable = Some(true);
                }
                Keyword::Not => {
                    self.next_token();
                    self.next_expected_keyword(Keyword::Null)?;
                    column.nullable = Some(false);
                }
                Keyword::Default => {
                    self.next_token();
                    self.next_token();
                    column.default = Some(self.parse_expression(Precedence::Lowest)?)
                }
                Keyword::Unique => {
                    self.next_token();
                    column.unique = true
                }
                Keyword::Index => {
                    self.next_token();
                    column.index = true
                }
                Keyword::References => {
                    self.next_token();
                    column.references = Some(self.next_ident()?)
                }
                keyword => {
                    return Err(Error::Parse(fmt_err!("unexpected keyword: {}", keyword)));
                }
            }
        }

        Ok(column)
    }

    fn parse_drop_stmt(&mut self) -> Result<Statement> {
        self.next_expected_keyword(Keyword::Table)?;
        Ok(Statement::DropTable(DropTableStmt {
            table_name: self.next_ident()?,
        }))
    }

    fn parse_select_stmt(&mut self) -> Result<Statement> {
        // SELECT [selects] [froms] [wheres] [group_by]
        //        [having] [order] [limit] [offset];
        Ok(Statement::Select(SelectStmt {
            selects: self.parse_clause_select()?,
            froms: {
                let froms_exprs = self.parse_clause_from()?;
                if froms_exprs.is_empty() {
                    None
                } else {
                    Some(froms_exprs)
                }
            },
            wheres: self.parse_clause_where()?,
            group_by: match self.parse_clause_group_by() {
                Ok(group_by_expr) => {
                    if group_by_expr.is_empty() {
                        None
                    } else {
                        Some(group_by_expr)
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            },
            having: self.parse_clause_having()?,
            order: {
                let order_exprs = self.parse_clause_order()?;
                if order_exprs.is_empty() {
                    None
                } else {
                    Some(order_exprs)
                }
            },
            offset: {
                match &self.pre_token {
                    Token::KeyWord(Keyword::Offset) => {
                        self.next_token();
                        Some(
                            self.parse_expression(Precedence::Lowest)
                                .map_err(|_| Error::Parse(fmt_err!("OFFSET exp is not valid!")))?,
                        )
                    }
                    _ => None,
                }
            },
            limit: {
                match &self.peek_token {
                    Token::KeyWord(Keyword::Limit) => {
                        self.next_token();
                        self.next_token();
                        Some(
                            self.parse_expression(Precedence::Lowest)
                                .map_err(|_| Error::Parse(fmt_err!("LIMIT exp is not valid!")))?,
                        )
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
                self.next_token();
                break;
            }

            let expr = self
                .parse_expression(Precedence::Lowest)
                .map_err(|_| Error::Parse(fmt_err!("SELECT expression is not valid!")))?;

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
                            return Err(Error::Parse(fmt_err!("AS is not valid!")));
                        }
                    }
                }
                Token::Ident(ident) => {
                    self.next_token();
                    Some(ident)
                }
                _ => None,
            };

            selects.push((expr, alias));

            self.next_token();
            match &self.pre_token {
                Token::Comma => continue,
                _ => break,
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
            loop {
                // SELECT t1.xxx, t2.xxx FROM t1 AS t3
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

                        Some(self.parse_expression(Precedence::Lowest).map_err(|_| {
                            Error::Parse(fmt_err!("ON Predicate expression is not valid!"))
                        })?)
                    }
                };

                item = FromItem::Join {
                    left: left_exp,
                    right: right_exp,
                    join_type,
                    predicate,
                };
            }
            froms.push(item);

            match &self.pre_token {
                Token::KeyWord(k) => match k {
                    Keyword::Where
                    | Keyword::Group
                    | Keyword::Having
                    | Keyword::Order
                    | Keyword::Limit
                    | Keyword::Offset => {
                        break;
                    }
                    _ => {}
                },
                _ => {
                    self.next_token();
                    break;
                }
            }
        }

        Ok(froms)
    }

    fn parse_clause_from_table(&mut self) -> Result<FromItem> {
        let name = match self.peek_token.clone() {
            Token::Ident(ident) => {
                self.next_token();
                ident
            }
            _ => {
                return Err(Error::Parse(fmt_err!("FROM table_name is not valid!")));
            }
        };

        let alias = match self.peek_token.clone() {
            Token::KeyWord(Keyword::As) => {
                self.next_token();
                match self.peek_token.clone() {
                    Token::Ident(ident) => {
                        self.next_token();
                        Some(ident)
                    }
                    _ => {
                        return Err(Error::Parse(fmt_err!("FROM AS is not valid!")));
                    }
                }
            }
            Token::Ident(ident) => {
                self.next_token();
                Some(ident)
            }
            _ => None,
        };

        Ok(FromItem::Table { name, alias })
    }

    fn parse_clause_from_jointype(&mut self) -> Result<Option<JoinType>> {
        match &self.peek_token {
            Token::KeyWord(Keyword::Outer) => {
                self.next_token();
                match self.peek_token.clone() {
                    Token::KeyWord(Keyword::Join) => {
                        self.next_token();

                        Ok(Some(JoinType::Outer))
                    }
                    _ => Ok(None),
                }
            }
            Token::KeyWord(Keyword::Inner) => {
                self.next_token();
                match self.peek_token.clone() {
                    Token::KeyWord(Keyword::Join) => {
                        self.next_token();

                        Ok(Some(JoinType::Inner))
                    }
                    _ => Ok(None),
                }
            }
            Token::KeyWord(Keyword::Left) => {
                self.next_token();
                match self.peek_token.clone() {
                    Token::KeyWord(Keyword::Outer) => {
                        self.next_token();
                        match self.peek_token.clone() {
                            Token::KeyWord(Keyword::Join) => {
                                self.next_token();

                                Ok(Some(JoinType::Left))
                            }
                            _ => Ok(None),
                        }
                    }
                    Token::KeyWord(Keyword::Join) => {
                        self.next_token();
                        Ok(Some(JoinType::Left))
                    }
                    _ => Ok(None),
                }
            }
            Token::KeyWord(Keyword::Right) => {
                self.next_token();
                match self.peek_token.clone() {
                    Token::KeyWord(Keyword::Outer) => {
                        self.next_token();
                        match self.peek_token.clone() {
                            Token::KeyWord(Keyword::Join) => {
                                self.next_token();

                                Ok(Some(JoinType::Right))
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
            // postgresql 和 sqlite 默认join 都是 inner join
            Token::KeyWord(Keyword::Join) => {
                self.next_token();
                Ok(Some(JoinType::Inner))
            }
            _ => Ok(None),
        }
    }

    fn parse_clause_group_by(&mut self) -> Result<Vec<Expression>> {
        let mut exprs = Vec::new();

        match &self.pre_token {
            Token::KeyWord(Keyword::Group) => {}
            _ => {
                return Ok(exprs);
            }
        }

        self.next_expected_keyword(Keyword::By)?;
        self.next_token();

        loop {
            exprs.push(
                self.parse_expression(Precedence::Lowest)
                    .map_err(|_| Error::Parse(fmt_err!("GROUP BY exp is not valid!")))?,
            );

            self.next_token();
            match self.peek_token {
                Token::Comma => continue,
                _ => break,
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

        return Ok(Some({
            let exp = self
                .parse_expression(Precedence::Lowest)
                .map_err(|_| Error::Parse(fmt_err!("WHERE exp is not valid!")))?;
            self.next_token();
            exp
        }));
    }

    fn parse_clause_having(&mut self) -> Result<Option<Expression>> {
        match self.pre_token {
            Token::KeyWord(Keyword::Having) => {
                self.next_token();
                Ok(Some({
                    let exp = self
                        .parse_expression(Precedence::Lowest)
                        .map_err(|_| Error::Parse(fmt_err!("HAVING exp is not valid!")))?;

                    self.next_token();
                    exp
                }))
            }
            _ => Ok(None),
        }
    }

    fn parse_clause_order(&mut self) -> Result<Vec<(Expression, OrderByType)>> {
        match &self.pre_token {
            Token::KeyWord(Keyword::Order) => {}
            _ => {
                return Ok(Vec::new());
            }
        }
        self.next_expected_keyword(Keyword::By)?;
        self.next_token();
        let mut orders = Vec::new();

        loop {
            orders.push((
                {
                    let exp = self
                        .parse_expression(Precedence::Lowest)
                        .map_err(|_| Error::Parse(fmt_err!("ORDER BY exp is not valid!")))?;
                    self.next_token();
                    exp
                },
                match self.pre_token {
                    Token::KeyWord(Keyword::Asc) => OrderByType::Asc,
                    Token::KeyWord(Keyword::Desc) => OrderByType::Desc,
                    _ => OrderByType::Asc,
                },
            ));
            if !self.next_if_token(Token::Comma) {
                break;
            }
        }

        Ok(orders)
    }

    fn parse_update_stmt(&mut self) -> Result<Statement> {
        let table_name = self.next_ident()?;

        self.next_expected_keyword(Keyword::Set)?;

        let mut set = BTreeMap::new();

        loop {
            let column = self.next_ident()?;
            self.next_expected_token(Token::Equal)?;
            self.next_token();
            let expr = self
                .parse_expression(Precedence::Lowest)
                .map_err(|_| Error::Parse(fmt_err!("expr is not valid!")))?;

            if set.contains_key(&column) {
                return Err(Error::Parse(fmt_err!(
                    "Duplicate values given for column {}",
                    column
                )));
            }
            set.insert(column, expr);
            if self.peek_token != Token::Comma {
                self.next_token();
                break;
            }
        }
        Ok(Statement::Update(UpdateStmt {
            table_name,
            set,
            wheres: self.parse_clause_where()?,
        }))
    }

    fn parse_explain_stmt(&mut self) -> Result<Statement> {
        self.next_token();
        Ok(Statement::Explain(ExplainStmt {
            statement: Box::new(self.parse_stmt()?),
        }))
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
            Err(Error::Parse(fmt_err!(
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
            Err(Error::Parse(fmt_err!(
                "unexpected token: {} want: {}",
                token,
                t
            )))
        }
    }

    fn next_ident(&mut self) -> Result<String> {
        match self.next_token() {
            Token::Ident(ident) => Ok(ident.clone()),
            t => Err(Error::Parse(fmt_err!(
                "expected: Token::Ident but get: {}",
                t
            ))),
        }
    }

    // (1 + 2)
    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression> {
        if !is_prefix_oper(&self.pre_token) {
            return Err(Error::Parse(fmt_err!(
                "No prefix Operator Func for: {:?}",
                &self.pre_token
            )));
        }

        let mut lhs = match self.parse_prefix_expr()? {
            Some(exp) => exp,
            None => {
                return Err(Error::Parse(fmt_err!("ParsePrefixExpression exp is None")));
            }
        };

        while self.pre_token != Token::Semicolon && precedence < self.peek_token_predence() {
            if !is_infix_oper(&self.peek_token) {
                return Ok(lhs);
            }
            self.next_token();
            lhs = self.parse_infix_expr(lhs)?;
        }

        Ok(lhs)
    }

    fn parse_prefix_expr(&mut self) -> Result<Option<Expression>> {
        // 1 + 2 + 3
        match self.pre_token.clone() {
            Token::Exclamation => {
                self.next_token();

                Ok(Some(Expression::Operation(Operation::Not(Box::new(
                    self.parse_expression(Precedence::Prefix)
                        .map_err(|_| Error::Parse(fmt_err!("Operation::Not exp is not valid!")))?,
                )))))
            }
            Token::Add => {
                self.next_token();
                Ok(Some(Expression::Operation(Operation::Assert(Box::new(
                    self.parse_expression(Precedence::Prefix).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::Assert exp is not valid!"))
                    })?,
                )))))
            }
            Token::Minus => {
                self.next_token();
                Ok(Some(Expression::Operation(Operation::Negate(Box::new(
                    self.parse_expression(Precedence::Prefix).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::Negate exp is not valid!"))
                    })?,
                )))))
            }
            Token::Number(n) => {
                // 如果包含 '.' 则说明是一个浮点数
                if n.contains('.') {
                    Ok(Some(Expression::Literal(Literal::Float(
                        match n.parse::<f64>() {
                            Ok(n) => n,
                            Err(e) => {
                                return Err(Error::Parse(fmt_err!("{}", e)));
                            }
                        },
                    ))))
                } else {
                    Ok(Some(Expression::Literal(Literal::Int(
                        match n.parse::<i64>() {
                            Ok(n) => n,
                            Err(e) => {
                                return Err(Error::Parse(fmt_err!("{}", e)));
                            }
                        },
                    ))))
                }
            }
            Token::LeftParen => {
                self.next_token();
                let exp = self.parse_expression(Precedence::Lowest)?;
                if !self.peek_if_token(Token::RightParen) {
                    return Ok(None);
                }

                Ok(Some(exp))
            }
            Token::Ident(i) => match &self.peek_token {
                Token::LeftParen => Ok(Some(Expression::Literal(Literal::String(i)))),
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
                                return Err(Error::Parse(fmt_err!("expected: Token::Ident")));
                            }
                        },
                    )))
                }
                _ => Ok(Some(Expression::Field(None, i))),
            },
            Token::String(s) => Ok(Some(Expression::Literal(Literal::String(s)))),
            Token::KeyWord(k) => match k {
                Keyword::True => Ok(Some(Expression::Literal(Literal::Bool(true)))),
                Keyword::False => Ok(Some(Expression::Literal(Literal::Bool(false)))),
                Keyword::Null => Ok(Some(Expression::Literal(Literal::Null))),
                _ => Err(Error::Parse(fmt_err!(
                    "No prefixOperatorFunc for {}",
                    self.pre_token
                ))),
            },

            _ => Err(Error::Parse(fmt_err!(
                "No prefixOperatorFunc for {}",
                self.pre_token
            ))),
        }
    }

    fn parse_infix_expr(&mut self, exp: Expression) -> Result<Expression> {
        match self.pre_token {
            Token::Add => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();
                Ok(Expression::Operation(Operation::Add(
                    Box::new(exp),
                    Box::new(
                        self.parse_expression(precedence).map_err(|_| {
                            Error::Parse(fmt_err!("Operation::Add exp is not valid!"))
                        })?,
                    ),
                )))
            }
            Token::Equal => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();

                Ok(Expression::Operation(Operation::Equal(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::Equal exp is not valid!"))
                    })?),
                )))
            }
            Token::GreaterThan => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();

                Ok(Expression::Operation(Operation::GreaterThan(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::GreaterThan exp is not valid!"))
                    })?),
                )))
            }
            Token::GreaterThanOrEqual => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();

                Ok(Expression::Operation(Operation::GreaterThanOrEqual(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::GreaterThanOrEqual is not valid!"))
                    })?),
                )))
            }
            Token::LessThan => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();

                Ok(Expression::Operation(Operation::LessThan(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::LessThan exp is not valid!"))
                    })?),
                )))
            }
            Token::LessThanOrEqual => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();

                Ok(Expression::Operation(Operation::LessThanOrEqual(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::LessThanOrEqual exp is not valid!"))
                    })?),
                )))
            }
            Token::Minus => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();

                Ok(Expression::Operation(Operation::Subtract(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::Minus exp is not valid!"))
                    })?),
                )))
            }
            Token::NotEqual => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();

                Ok(Expression::Operation(Operation::NotEqual(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::NotEqual exp is not valid!"))
                    })?),
                )))
            }
            Token::KeyWord(Keyword::And) => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();

                Ok(Expression::Operation(Operation::And(
                    Box::new(exp),
                    Box::new(
                        self.parse_expression(precedence).map_err(|_| {
                            Error::Parse(fmt_err!("Operation::And exp is not valid!"))
                        })?,
                    ),
                )))
            }
            Token::KeyWord(Keyword::Or) => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();

                Ok(Expression::Operation(Operation::Or(
                    Box::new(exp),
                    Box::new(
                        self.parse_expression(precedence).map_err(|_| {
                            Error::Parse(fmt_err!("Operation::Or exp is not valid!"))
                        })?,
                    ),
                )))
            }
            Token::KeyWord(Keyword::Like) => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();

                Ok(Expression::Operation(Operation::Like(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::Like exp is not valid!"))
                    })?),
                )))
            }
            Token::Percent => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();
                Ok(Expression::Operation(Operation::Modulo(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::Percent exp is not valid!"))
                    })?),
                )))
            }
            Token::Asterisk => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();
                Ok(Expression::Operation(Operation::Multiply(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::Asterisk exp is not valid!"))
                    })?),
                )))
            }
            Token::Slash => {
                let precedence = match_precedence(&self.pre_token);
                self.next_token();
                Ok(Expression::Operation(Operation::Divide(
                    Box::new(exp),
                    Box::new(self.parse_expression(precedence).map_err(|_| {
                        Error::Parse(fmt_err!("Operation::Slash exp is not valid!"))
                    })?),
                )))
            }
            // 如果 ( 是一个中缀运算符, 则是一个函数
            Token::LeftParen => Ok(Expression::Function(
                match exp {
                    Expression::Literal(Literal::String(s)) => s,
                    _ => {
                        return Err(Error::Parse(fmt_err!(
                            "Operation::LeftParen exp is not Literal::String"
                        )));
                    }
                },
                match self.peek_token {
                    // SELECT FUNCTION_NAME(*)
                    Token::Asterisk => {
                        self.next_token();
                        match self.peek_token {
                            Token::RightParen => {
                                self.next_token();
                                vec![Expression::Literal(Literal::All)]
                            }
                            _ => {
                                return Err(Error::Parse(fmt_err!(
                                    "Operation::LeftParen exp is not Literal::String"
                                )));
                            }
                        }
                    }
                    Token::RightParen => {
                        // empty function args, like SUM(), NOW()
                        self.next_token();
                        vec![]
                    }
                    _ => match self.parse_expression_list()? {
                        Some(exprs) => exprs,
                        None => {
                            return Err(Error::Parse(fmt_err!("Operation::LeftParen exp is None")));
                        }
                    },
                },
            )),
            _ => Err(Error::Parse(fmt_err!(
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
        exprs.push(
            self.parse_expression(Precedence::Lowest)
                .map_err(|_| Error::Parse(fmt_err!("Operation::LeftParen exp is not valid!")))?,
        );

        while self.peek_if_token(Token::Comma) {
            self.next_token();

            exprs.push(
                self.parse_expression(Precedence::Lowest).map_err(|_| {
                    Error::Parse(fmt_err!("parse_expression_list exp is not valid!"))
                })?,
            );
        }

        if !self.peek_if_token(Token::RightParen) {
            return Ok(None);
        }

        Ok(Some(exprs))
    }

    fn peek_token_predence(&self) -> Precedence {
        operator::match_precedence(&self.peek_token)
    }
}

#[cfg(test)]
pub mod test {

    macro_rules! test_parser {
        ( $($name:ident: $sql:expr => $except:expr,  )* ) => {
            $(
                #[test]
                fn $name() {
                    init();
                    let mut parser = Parser::new_parser($sql.to_owned());
                    let result = parser.parse_stmt();
                    assert_eq!(result, $except);
                }
            )*
        };
    }

    use std::vec;

    use super::stmt::*;
    use super::*;
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

    test_parser! {
        explain_base_sql: "explain drop table person" => Ok(Statement::Explain(ExplainStmt {
            statement: Box::new(Statement::DropTable(DropTableStmt {
                table_name: "person".to_owned(),
            })),
        })),
        update_table_base: "update person set name = 'tangruilin' where id = 1;" => Ok(Statement::Update(UpdateStmt {
            table_name: "person".to_owned(),
            set: BTreeMap::from([(
                "name".to_owned(),
                Expression::Literal(Literal::String(
                    "tangruilin".to_owned(),
                )),
            )]),
            wheres: Some(Expression::Operation(Operation::Equal(
                Box::new(Expression::Field(None, "id".to_owned())),
                Box::new(Expression::Literal(Literal::Int(1))),
            ))),
        })),
        drop_table_base: "drop table person;" => Ok(Statement::DropTable(DropTableStmt {
            table_name: "person".to_owned(),
        })),
        delete_table_base: "delete from person where id = 1;" => Ok(Statement::Delete(DeleteTableStmt {
            table_name: "person".to_owned(),
            r#where: Some(Expression::Operation(Operation::Equal(
                Box::new(Expression::Field(None, "id".to_owned())),
                Box::new(Expression::Literal(Literal::Int(1))),
            ))),
        })),
        delete_table_without_where: "delete from person;" => Ok(Statement::Delete(DeleteTableStmt {
            table_name: "person".to_owned(),
            r#where: None,
        })),
        insert_table_base: "insert into person (id, name, age) values (1, 'tangruilin', 14)" => Ok(Statement::Insert(InsertStmt {
                table_name: "person".to_owned(),
                columns: Some(["id".to_owned(), "name".to_owned(), "age".to_owned()].to_vec()),
                values: [[
                    Some(Expression::Literal(Literal::Int(1))),
                    Some(Expression::Literal(Literal::String(
                        "tangruilin".to_owned()
                    ))),
                    Some(Expression::Literal(Literal::Int(14))),
                ]
                .to_vec()]
                .to_vec(),
            })),

        insert_without_column_name: "insert into person values (1, 'tangruilin', 14)" => Ok(Statement::Insert(InsertStmt {
                table_name: "person".to_owned(),
                columns: None,
                values: [[
                    Some(Expression::Literal(Literal::Int(1))),
                    Some(Expression::Literal(Literal::String(
                        "tangruilin".to_owned()
                    ))),
                    Some(Expression::Literal(Literal::Int(14))),
                ]
                .to_vec()]
                .to_vec(),
            })),

        create_table_success: "create table person (id int primary key, name string not null default 'tangruilin', age int unique, class int index references country);" => Ok(Statement::CreateTable(stmt::CreateTableStmt {
            columns: vec![
                column::Column {
                    name: "id".to_string(),
                    data_type: DataType::Int32,
                    primary_key: true,
                    nullable: None,
                    default: None,
                    unique: false,
                    index: false,
                    references: None,
                },
                column::Column {
                    name: "name".to_string(),
                    data_type: DataType::String,
                    primary_key: false,
                    nullable: Some(false),
                    default: Some(Expression::Literal(Literal::String(
                        "tangruilin".to_owned(),
                    ))),
                    unique: false,
                    index: false,
                    references: None,
                },
                column::Column {
                    name: "age".to_string(),
                    data_type: DataType::Int32,
                    primary_key: false,
                    nullable: None,
                    default: None,
                    unique: true,
                    index: false,
                    references: None,
                },
                column::Column {
                    name: "class".to_string(),
                    data_type: DataType::Int32,
                    primary_key: false,
                    nullable: None,
                    default: None,
                    unique: false,
                    index: true,
                    references: Some("country".to_owned()),
                },
            ],
            table_name: "person".to_string(),
        })),
        transaction_begin_transaction: "begin transaction;" => Ok(Statement::Begin(BeginStmt {
            is_readonly: false,
            version: None,
        })),
        transaction_read_only_begin: "begin transaction read only;" => Ok(Statement::Begin(BeginStmt {
            is_readonly: true,
            version: None,
        })),
        transaction_begin: "begin;" => Ok(Statement::Begin(BeginStmt {
            is_readonly: false,
            version: None,
        })),
        transaction_commit: "commit;" => Ok(Statement::Commit),
        transaction_rollback: "rollback" => Ok(Statement::Rollback),
        transaction_read_write: "BEGIN TRANSACTION READ WRITE" => Ok(Statement::Begin(BeginStmt {
            is_readonly: false,
            version: None,
        })),
        transaction_read_only_with_version: "BEGIN TRANSACTION READ ONLY AS OF SYSTEM TIME 129012313;" => Ok( Statement::Begin(BeginStmt {
            is_readonly: true,
            version: Some(129012313),
        })),
        select_base: "SELECT c1 AS c2 FROM table_1;" => Ok(Statement::Select(SelectStmt {
            selects: vec![(
                Expression::Field(None, "c1".to_owned()),
                Some("c2".to_owned()),
            )],
            froms: Some(vec![FromItem::Table {
                name: "table_1".to_owned(),
                alias: None,
            }]),
            wheres: None,
            group_by: None,
            having: None,
            order: None,
            limit: None,
            offset: None,
        })),
        select_with_middle_complex: r#"SELECT 1 + 2 AS c1, user.id FROM table_1 AS table_2
                                LEFT JOIN table_3 AS table_4
                                ON table_2.id = table_4.id
                                ORDER BY table_2.id ASC OFFSET 10;"# => Ok(Statement::Select(SelectStmt {
            selects: vec![
                ((
                    Expression::Operation(Operation::Add(
                        Box::new(Expression::Literal(Literal::Int(1))),
                        Box::new(Expression::Literal(Literal::Int(2))),
                    )),
                    Some("c1".to_owned()),
                )),
                (
                    Expression::Field(Some("user".to_owned()), "id".to_owned()),
                    None,
                ),
            ],
            froms: Some(vec![FromItem::Join {
                left: Box::new(FromItem::Table {
                    name: "table_1".to_owned(),
                    alias: Some("table_2".to_owned()),
                }),
                right: Box::new(FromItem::Table {
                    name: "table_3".to_owned(),
                    alias: Some("table_4".to_owned()),
                }),
                join_type: JoinType::Left,
                predicate: Some(Expression::Operation(Operation::Equal(
                    Box::new(Expression::Field(
                        Some("table_2".to_owned()),
                        "id".to_owned(),
                    )),
                    Box::new(Expression::Field(
                        Some("table_4".to_owned()),
                        "id".to_owned(),
                    )),
                ))),
            }]),
            wheres: None,
            group_by: None,
            having: None,
            order: Some(vec![(
                Expression::Field(Some("table_2".to_owned()), "id".to_owned()),
                OrderByType::Asc,
            )]),
            offset: Some(Expression::Literal(Literal::Int(10))),
            limit: None,
        })),
        select_with_high_complex: r#"SELECT c.category_name, COUNT(p.product_id) AS product_count, AVG(p.unit_price) AS avg_price
                 FROM categories c
                 LEFT JOIN products p ON c.category_id = p.category_id
                 RIGHT JOIN orders o ON p.product_id = o.product_id
                 WHERE o.order_date >= '2023-01-01' AND o.order_date <= '2023-12-31'
                 GROUP BY c.category_name
                 HAVING COUNT(p.product_id) >= 5
                 ORDER BY avg_price DESC OFFSET 4 + 10 * 10.1 LIMIT 3;"# => Ok(Statement::Select(SelectStmt {
            selects: vec![
                (
                    Expression::Field(Some("c".to_owned()), "category_name".to_owned()),
                    None,
                ),
                (
                    Expression::Function(
                        "COUNT".to_owned(),
                        vec![Expression::Field(
                            Some("p".to_owned()),
                            "product_id".to_owned(),
                        )],
                    ),
                    Some("product_count".to_owned()),
                ),
                (
                    Expression::Function(
                        "AVG".to_owned(),
                        vec![Expression::Field(
                            Some("p".to_owned()),
                            "unit_price".to_owned(),
                        )],
                    ),
                    Some("avg_price".to_owned()),
                ),
            ],
            froms: Some(vec![FromItem::Join {
                left: Box::new(FromItem::Join {
                    left: Box::new(FromItem::Table {
                        name: "categories".to_owned(),
                        alias: Some("c".to_owned()),
                    }),
                    right: Box::new(FromItem::Table {
                        name: "products".to_owned(),
                        alias: Some("p".to_owned()),
                    }),
                    join_type: JoinType::Left,
                    predicate: Some(Expression::Operation(Operation::Equal(
                        Box::new(Expression::Field(
                            Some("c".to_owned()),
                            "category_id".to_owned(),
                        )),
                        Box::new(Expression::Field(
                            Some("p".to_owned()),
                            "category_id".to_owned(),
                        )),
                    ))),
                }),
                right: Box::new(FromItem::Table {
                    name: "orders".to_owned(),
                    alias: Some("o".to_owned()),
                }),
                join_type: JoinType::Right,
                predicate: Some(Expression::Operation(Operation::Equal(
                    Box::new(Expression::Field(
                        Some("p".to_owned()),
                        "product_id".to_owned(),
                    )),
                    Box::new(Expression::Field(
                        Some("o".to_owned()),
                        "product_id".to_owned(),
                    )),
                ))),
            }]),
            wheres: Some(Expression::Operation(Operation::And(
                Box::new(Expression::Operation(Operation::GreaterThanOrEqual(
                    Box::new(Expression::Field(
                        Some("o".to_owned()),
                        "order_date".to_owned(),
                    )),
                    Box::new(Expression::Literal(Literal::String(
                        "2023-01-01".to_owned(),
                    ))),
                ))),
                Box::new(Expression::Operation(Operation::LessThanOrEqual(
                    Box::new(Expression::Field(
                        Some("o".to_owned()),
                        "order_date".to_owned(),
                    )),
                    Box::new(Expression::Literal(Literal::String(
                        "2023-12-31".to_owned(),
                    ))),
                ))),
            ))),
            group_by: Some(vec![Expression::Field(
                Some("c".to_owned()),
                "category_name".to_owned(),
            )]),
            having: Some(Expression::Operation(Operation::GreaterThanOrEqual(
                Box::new(Expression::Function(
                    "COUNT".to_owned(),
                    vec![Expression::Field(
                        Some("p".to_owned()),
                        "product_id".to_owned(),
                    )],
                )),
                Box::new(Expression::Literal(Literal::Int(5))),
            ))),
            order: Some(vec![(
                Expression::Field(None, "avg_price".to_owned()),
                OrderByType::Desc,
            )]),
            offset: Some(Expression::Operation(Operation::Add(
                Box::new(Expression::Literal(Literal::Int(4))),
                Box::new(Expression::Operation(Operation::Multiply(
                    Box::new(Expression::Literal(Literal::Int(10))),
                    Box::new(Expression::Literal(Literal::Float(10.1))),
                ))),
            ))),
            limit: Some(Expression::Literal(Literal::Int(3))),
        })),
        select_with_simple_expression: r#"SELECT 1 + 2 AS c1, account.id
                 FROM table_1
                 OFFSET TRUE AND FALSE
                 LIMIT 10;
                "# => Ok(Statement::Select(SelectStmt {
            selects: vec![
                (
                    Expression::Operation(Operation::Add(
                        Box::new(Expression::Literal(Literal::Int(1))),
                        Box::new(Expression::Literal(Literal::Int(2))),
                    )),
                    Some("c1".to_owned()),
                ),
                (
                    Expression::Field(Some("account".to_owned()), "id".to_owned()),
                    None,
                ),
            ],
            froms: Some(vec![FromItem::Table {
                name: "table_1".to_owned(),
                alias: None,
            }]),
            wheres: None,
            group_by: None,
            having: None,
            order: None,
            offset: Some(Expression::Operation(Operation::And(
                Box::new(Expression::Literal(Literal::Bool(true))),
                Box::new(Expression::Literal(Literal::Bool(false))),
            ))),
            limit: Some(Expression::Literal(Literal::Int(10))),
        })),
        select_with_alias: r#"SELECT c1.id FROM b2 AS c1 ORDER BY c1.id;"# => Ok(Statement::Select(SelectStmt {
            selects: vec![(
                Expression::Field(Some("c1".to_owned()), "id".to_owned()),
                None,
            )],
            froms: Some(vec![FromItem::Table {
                name: "b2".to_owned(),
                alias: Some("c1".to_owned()),
            }]),
            wheres: None,
            group_by: None,
            having: None,
            order: Some(vec![(
                Expression::Field(Some("c1".to_owned()), "id".to_owned()),
                OrderByType::Asc,
            )]),
            offset: None,
            limit: None,
        })),
        select_with_aggression: r#"SELECT COUNT(*) FROM user WHERE user.id != NULL;"# => Ok(Statement::Select(SelectStmt {
            selects: vec![(
                Expression::Function("COUNT".to_owned(), vec![Expression::Literal(Literal::All)]),
                None,
            )],
            froms: Some(vec![FromItem::Table {
                name: "user".to_owned(),
                alias: None,
            }]),
            wheres: Some(Expression::Operation(Operation::NotEqual(
                Box::new(Expression::Field(Some("user".to_owned()), "id".to_owned())),
                Box::new(Expression::Literal(Literal::Null)),
            ))),
            group_by: None,
            having: None,
            order: None,
            offset: None,
            limit: None,
        })),
        select_with_join: "SELECT 1 + 2 AS c1, c3.id FROM c5 JOIN c6 ON c5.id = c6.id;" => Ok( Statement::Select(SelectStmt {
            selects: vec![
                (
                    Expression::Operation(Operation::Add(
                        Box::new(Expression::Literal(Literal::Int(1))),
                        Box::new(Expression::Literal(Literal::Int(2))),
                    )),
                    Some("c1".to_owned()),
                ),
                (
                    Expression::Field(Some("c3".to_owned()), "id".to_owned()),
                    None,
                ),
            ],
            froms: Some(vec![FromItem::Join {
                left: Box::new(FromItem::Table {
                    name: "c5".to_owned(),
                    alias: None,
                }),
                right: Box::new(FromItem::Table {
                    name: "c6".to_owned(),
                    alias: None,
                }),
                join_type: JoinType::Inner,
                predicate: Some(Expression::Operation(Operation::Equal(
                    Box::new(Expression::Field(Some("c5".to_owned()), "id".to_owned())),
                    Box::new(Expression::Field(Some("c6".to_owned()), "id".to_owned())),
                ))),
            }]),
            wheres: None,
            group_by: None,
            having: None,
            order: None,
            offset: None,
            limit: None,
        })),
        select_with_aggression_and_alias: r#"SELECT COUNT(*) AS c1, AVG(test_1.id) AS c2, 1 + 2 * (-10) AS c3
                 FROM test_1 WHERE c1.id = -10;"# => Ok(Statement::Select(SelectStmt {
            selects: vec![
                (
                    Expression::Function(
                        "COUNT".to_owned(),
                        vec![Expression::Literal(Literal::All)],
                    ),
                    Some("c1".to_owned()),
                ),
                (
                    Expression::Function(
                        "AVG".to_owned(),
                        vec![Expression::Field(
                            Some("test_1".to_owned()),
                            "id".to_owned(),
                        )],
                    ),
                    Some("c2".to_owned()),
                ),
                (
                    Expression::Operation(Operation::Add(
                        Box::new(Expression::Literal(Literal::Int(1))),
                        Box::new(Expression::Operation(Operation::Multiply(
                            Box::new(Expression::Literal(Literal::Int(2))),
                            Box::new(Expression::Operation(Operation::Negate(Box::new(
                                Expression::Literal(Literal::Int(10)),
                            )))),
                        ))),
                    )),
                    Some("c3".to_owned()),
                ),
            ],
            froms: Some(vec![FromItem::Table {
                name: "test_1".to_owned(),
                alias: None,
            }]),
            wheres: Some(Expression::Operation(Operation::Equal(
                Box::new(Expression::Field(Some("c1".to_owned()), "id".to_owned())),
                Box::new(Expression::Operation(Operation::Negate(Box::new(
                    Expression::Literal(Literal::Int(10)),
                )))),
            ))),
            group_by: None,
            having: None,
            order: None,
            offset: None,
            limit: None,
        }
        )),
        alter_table_test_1: r#"ALTER TABLE user ADD COLUMN password STRING DEFAULT 3 + 5;"#
        => Ok(Statement::Alter(AlterStmt {
            alter_type: AlterType::AddColumn(Column {
                name: "password".to_owned(),
                data_type: DataType::String,
                primary_key: false,
                nullable: None,
                default: Some(Expression::Operation(Operation::Add(
                    Box::new(Expression::Literal(Literal::Int(3))),
                    Box::new(Expression::Literal(Literal::Int(5))),
                ))),
                unique: false,
                index: false,
                references: None,
            }),
            table_name: "user".to_owned(),
        })),
        alter_table_test_2: r#"ALTER TABLE user ADD INDEX user_id_index (account, id);"# =>
            Ok(Statement::Alter(AlterStmt {
                alter_type: AlterType::AddIndex(
                    Some("user_id_index".to_owned()),
                    vec!["account".to_owned(), "id".to_owned()],
                ),
                table_name: "user".to_owned(),
            })),
        alter_table_test_3: r#"ALTER TABLE user DROP COLUMN password;"# =>
            Ok(Statement::Alter(AlterStmt {
                alter_type: AlterType::DropColumn("password".to_owned()),
                table_name: "user".to_owned(),
            })),
        alter_table_test_4: r#"ALTER TABLE user DROP INDEX user_id_index;"# =>
            Ok(Statement::Alter(AlterStmt {
                alter_type: AlterType::RemoveIndex("user_id_index".to_owned()),
                table_name: "user".to_owned(),
            })),
        alter_table_test_5: r#"ALTER TABLE user MODIFY COLUMN new_column_name INT PRIMARY KEY;"# =>
            Ok(Statement::Alter(AlterStmt {
                alter_type: AlterType::ModifyColumn(Column {
                    name: "new_column_name".to_owned(),
                    data_type: DataType::Int32,
                    primary_key: true,
                    nullable: None,
                    default: None,
                    unique: false,
                    index: false,
                    references: None,
                }),
                table_name: "user".to_owned(),
            })),
        alter_table_test_6: r#"ALTER TABLE user RENAME TO new_table_name;"# =>
            Ok(Statement::Alter(AlterStmt {
                alter_type: AlterType::RenameTable("new_table_name".to_owned()),
                table_name: "user".to_owned(),
            })),
        alter_table_test_7: r#"ALTER TABLE user RENAME COLUMN account TO account_2 "# =>
            Ok(Statement::Alter(AlterStmt {
                alter_type: AlterType::RenameColumn (
                    "account".to_owned(),
                    "account_2".to_owned(),
                ),
                table_name: "user".to_owned(),
            })),
        create_index_test_1: r#"CREATE INDEX xxx_name ON table_name (id, password, account);"# =>
            Ok(Statement::CreateIndex(CreateIndexStmt {
                is_unique: false,
                index_name: "xxx_name".to_owned(),
                table_name: "table_name".to_owned(),
                columns: vec!["id".to_owned(), "password".to_owned(), "account".to_owned()],
            })),
        create_unique_index_test2: r#"CREATE UNIQUE INDEX index_name ON table_name (id, password);"# =>
            Ok(Statement::CreateIndex(CreateIndexStmt {
                is_unique: true,
                index_name: "index_name".to_owned(),
                table_name: "table_name".to_owned(),
                columns: vec!["id".to_owned(), "password".to_owned()],
            })),
        set_transaction_test_1: r#"SET SESSION TRANSACTION ISOLATION LEVEL READ UNCOMMITTED;"# =>
            Ok(Statement::Set(SetStmt {
                set_value: SetVariableType::Transaction(TransactionIsolationLevel::ReadUncommitted),
                is_session: true,
            })),
        set_transaction_test_2: r#"SET TRANSACTION ISOLATION LEVEL READ COMMITTED;"# =>
            Ok(Statement::Set(SetStmt {
                set_value: SetVariableType::Transaction(TransactionIsolationLevel::ReadCommitted),
                is_session: true,
            })),
        set_transaction_test_3: r#"SET GLOBAL TRANSACTION ISOLATION LEVEL REPEATABLE READ;"# =>
            Ok(Statement::Set(SetStmt {
                set_value: SetVariableType::Transaction(TransactionIsolationLevel::RepeatableRead),
                is_session: false,
            })),
        set_transaction_test_4: r#"SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;"# =>
            Ok(Statement::Set(SetStmt {
                set_value: SetVariableType::Transaction(TransactionIsolationLevel::Serializable),
                is_session: true,
            })),

        show_databases_test: r#"SHOW DATABASES;"# =>
            Ok(Statement::ShowDatabase),
        show_tables_test: r#"SHOW TABLES;"# =>
            Ok(Statement::ShowTables),
    }
}
