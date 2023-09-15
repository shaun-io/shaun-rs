use super::expression::Expression;
use super::operation::Operation;
use super::token::Token;

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

pub fn match_precedence(t: Token) -> Precedence {
    match t {
        Token::Equal => Precedence::Equals,
        Token::NotEqual => Precedence::Equals,
        Token::LessThan => Precedence::LessGreater,
        Token::LessThanOrEqual => Precedence::LessGreater,
        Token::GreaterThan => Precedence::LessGreater,
        Token::GreaterThanOrEqual => Precedence::LessGreater,
        Token::Add => Precedence::Sum,
        Token::Minus => Precedence::Sum,
        Token::Slash => Precedence::Product,
        Token::Asterisk => Precedence::Product,
        Token::LeftParen => Precedence::Call,
        _ => Precedence::Lowest,
    }
}
