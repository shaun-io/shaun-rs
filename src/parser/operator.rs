use super::keyword::Keyword;
use super::token::{self, Token};

#[derive(Eq, PartialEq, Debug, PartialOrd, Ord)]
pub enum Precedence {
    Lowest,      // 最低优先级
    Equals,      // ==
    LessGreater, // > or < or >= or <=
    Sum,         // +
    Product,     // * /
    Prefix,      // -X or !X
    Call,        // function(x)
}

pub fn is_prefix_oper(t: &token::Token) -> bool {
    match t {
        Token::Exclamation => true,
        Token::Minus => true,
        Token::Add => true,
        Token::Number(_) => true,
        Token::LeftParen => true,
        Token::Ident(_) => true,
        Token::KeyWord(Keyword::True) => true,
        Token::KeyWord(Keyword::False) => true,
        _ => false,
    }
}

pub fn is_infix_oper(t: &token::Token) -> bool {
    match t {
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
        Token::KeyWord(Keyword::And) => true,
        Token::KeyWord(Keyword::Like) => true,
        Token::KeyWord(Keyword::Or) => true,
        Token::LeftParen => true,
        _ => false,
    }
}

pub fn match_precedence(t: Token) -> Precedence {
    match t {
        Token::Equal => Precedence::Equals,
        Token::NotEqual => Precedence::Equals,
        Token::LessThan => Precedence::LessGreater,
        Token::LessThanOrEqual => Precedence::LessGreater,
        Token::GreaterThan => Precedence::LessGreater,
        Token::GreaterThanOrEqual => Precedence::LessGreater,
        Token::KeyWord(k) => match k {
            Keyword::And => Precedence::Sum,
            Keyword::Or => Precedence::Sum,
            Keyword::Not => Precedence::Sum,
            _ => Precedence::Lowest,
        },
        Token::Add => Precedence::Sum,
        Token::Minus => Precedence::Sum,
        Token::Slash => Precedence::Product,
        Token::Asterisk => Precedence::Product,
        Token::LeftParen => Precedence::Call,
        _ => Precedence::Lowest,
    }
}
