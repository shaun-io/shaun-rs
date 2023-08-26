pub mod column;
mod data_type;
mod expression;
mod keyword;
mod lexer;
mod stmt;
pub mod token;

use lexer::Lexer;
use stmt::Statement;


pub struct Parser {
    lexer: lexer::Lexer,
}

impl Parser {
    pub fn new_parser(sql_str: String) -> Self {
        Parser {
            lexer: Lexer::new_lexer(sql_str),
        }
    }

    pub fn parse_stmt() -> Result<Statement, String> {
        
    }
}
