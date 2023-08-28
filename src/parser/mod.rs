pub mod column;
mod data_type;
mod expression;
mod keyword;
mod lexer;
mod stmt;
pub mod token;

use column::Column;
use keyword::Keyword;
use lexer::Lexer;
use stmt::Statement;
use token::Token;
use data_type::DataType;

use log::{debug, error};

pub struct Parser {
    lexer: lexer::Lexer,
    pre_token: token::Token,
}

impl Parser {
    pub fn new_parser(sql_str: String) -> Self {
        Parser {
            lexer: Lexer::new_lexer(sql_str),
            pre_token: token::Token::Eof,
        }
    }

    pub fn update(&mut self, sql_str: &str) {
        self.lexer.update(sql_str.to_owned());

        self.pre_token = token::Token::Eof;
    }

    pub fn parse_stmt(&mut self) -> Result<Statement, String> {
        // 直接与 lexer 产生的第一个 Token 作比较
        self.pre_token = self.lexer.next_token();
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

        self.next_token();

        if self.pre_token != Token::Eof && self.pre_token != Token::Semicolon {
            return Err(format!(
                "unexpected: {} want: {}",
                self.pre_token,
                Token::Eof
            ));
        }

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
        self.next_if_keyword(Keyword::Table);
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
        self.next_if_token(Token::LeftParen);

        let columns = Vec::new();
        loop {}

        Ok(Statement::CreateTable(CreateTableStmt {
            table_name: table_name,
        }))
    }

    fn parse_column(&mut self) -> Result<Column, String> {
        let name = self.next_ident();
        let mut column_name;

        match name {
            Ok(n) => {
                column_name = n;
            }
            Err(e) => {
                return Err(e);
            }
        }

        let column = column::Column {
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


        Err("e".to_string())
    }

    fn parse_drop_stmt(&mut self) -> Result<Statement, String> {
        unimplemented!()
    }
    fn parse_select_stmt(&mut self) -> Result<Statement, String> {
        unimplemented!()
    }
    fn parse_update_stmt(&mut self) -> Result<Statement, String> {
        unimplemented!()
    }
    fn parse_explain_stmt(&mut self) -> Result<Statement, String> {
        unimplemented!()
    }

    fn next_token(&mut self) -> &Token {
        self.pre_token = self.lexer.next_token();

        &self.pre_token
    }

    fn next_if_token(&mut self, t: Token) -> bool {
        *self.next_token() == t
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

    fn next_ident(&mut self) -> Result<String, String> {
        match self.next_token() {
            Token::Ident(ident) => Ok(ident.clone()),
            t => Err(format!("expected: Token::Ident but get: {}", t)),
        }
    }
}

#[cfg(test)]
mod test {

    use super::stmt::*;
    use super::*;

    #[test]
    fn init() {
        env_logger::init();
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
}
