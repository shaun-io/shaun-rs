use super::keyword::Keyword;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Clone)]
pub enum Token {
    Number(String),     // 数字
    String(String),     // 字符串, 'xxx' "xxx"
    Ident(String),      // 用户定义
    KeyWord(Keyword),   // 关键字
    Period,             // .
    Equal,              // =
    GreaterThan,        // >
    GreaterThanOrEqual, // >=
    LessThan,           // <
    LessThanOrEqual,    // <=
    Add,                // +
    Minus,              // -
    Asterisk,           // *
    Slash,              // /
    Caret,              // ^
    Percent,            // %
    Exclamation,        // !
    NotEqual,           // !=
    Question,           // ?
    LeftParen,          // (
    RightParen,         // )
    Comma,              // ,
    Semicolon,          // ;
    Eof,                // 语句结束
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token: {}",
            match self {
                Self::Number(number) => format!("Number {}", number),
                Self::String(string) => format!("String {}", string),
                Self::Ident(string) => format!("Ident: {}", string),
                Self::KeyWord(keyword) => format!("{}", keyword),
                Self::Period => "Period".to_string(),
                Self::Equal => "Equal".to_string(),
                Self::GreaterThan => "GreaterThan".to_string(),
                Self::LessThan => "LessThan".to_string(),
                Self::LessThanOrEqual => "LessThanOrEqual".to_string(),
                Self::Add => "Add".to_string(),
                Self::Minus => "Minus".to_string(),
                Self::Asterisk => "Asterisk".to_string(),
                Self::Slash => "Slash".to_string(),
                Self::Caret => "Caret".to_string(),
                Self::Percent => "Percent".to_string(),
                Self::Exclamation => "Exclamation".to_string(),
                Self::NotEqual => "NotEqual".to_string(),
                Self::Question => "Question".to_string(),
                Self::LeftParen => "LeftParen".to_string(),
                Self::RightParen => "RightParen".to_string(),
                Self::Comma => "Comma".to_string(),
                Self::Semicolon => "Semicolon".to_string(),
                Self::Eof => "Eof".to_string(),
                _ => {
                    "unknown".to_string()
                }
            }
        )
    }
}
