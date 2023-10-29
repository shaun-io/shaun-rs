use super::keyword::Keyword;
use super::token::{self, Token};

#[derive(Eq, PartialEq, Debug, PartialOrd, Ord)]
pub enum Precedence {
    Lowest,      // 最低优先级
    Equals,      // ==
    Logical,     // AND OR NOT 逻辑运算符
    LessGreater, // > or < or >= or <=
    Sum,         // +
    Product,     // * /
    Prefix,      // -X or !X
    Call,        // function(x)
}

#[warn(clippy::match_like_matches_macro)]
pub fn is_prefix_oper(t: &token::Token) -> bool {
    matches!(
        t,
        Token::Exclamation
            | Token::Minus
            | Token::Add
            | Token::LeftParen
            | Token::Number(_)
            | Token::Ident(_)
            | Token::KeyWord(Keyword::True)
            | Token::KeyWord(Keyword::False)
            | Token::String(_)
            | Token::KeyWord(Keyword::Null)
    )
}

pub fn is_infix_oper(t: &token::Token) -> bool {
    matches!(
        t,
        Token::Add
            | Token::Equal
            | Token::GreaterThan
            | Token::GreaterThanOrEqual
            | Token::LessThan
            | Token::LessThanOrEqual
            | Token::Minus
            | Token::NotEqual
            | Token::Percent
            | Token::Slash
            | Token::Asterisk
            | Token::Caret
            | Token::KeyWord(Keyword::And)
            | Token::KeyWord(Keyword::Like)
            | Token::KeyWord(Keyword::Or)
            | Token::LeftParen
    )
}

pub fn match_precedence(t: &Token) -> Precedence {
    match t {
        Token::Equal | Token::NotEqual => Precedence::Equals,
        Token::LessThan
        | Token::LessThanOrEqual
        | Token::GreaterThan
        | Token::GreaterThanOrEqual => Precedence::LessGreater,
        Token::KeyWord(Keyword::And)
        | Token::KeyWord(Keyword::Or)
        | Token::KeyWord(Keyword::Not) => Precedence::Logical,
        Token::Add | Token::Minus => Precedence::Sum,
        Token::Slash => Precedence::Product,
        Token::Asterisk => Precedence::Product,
        Token::LeftParen => Precedence::Call,
        _ => Precedence::Lowest,
    }
}
