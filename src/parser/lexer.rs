use super::keyword::Keyword;
use super::token::Token;
use crate::parser::keyword::find_keyword;

const STOP_CHAR: char = 0 as char;

pub struct Lexer {
    origin_str: String,
    cur_read_char: char,
    pos: usize,
    read_pos: usize,
}

impl Lexer {
    pub fn new_lexer(sql_str: String) -> Self {
        let mut lexer = Lexer {
            origin_str: sql_str,
            cur_read_char: STOP_CHAR,
            pos: 0,
            read_pos: 0,
        };
        lexer.read_char();

        lexer
    }

    pub fn update(&mut self, new_sql_str: String) -> &Self {
        self.origin_str = new_sql_str;
        self.cur_read_char = STOP_CHAR;
        self.pos = 0;
        self.read_pos = 0;
        self.read_char();

        self
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_space();

        let t = match self.cur_read_char {
            '=' => Token::Equal,
            '.' => Token::Period,
            '>' => match self.peek_char() {
                '=' => {
                    self.read_char();
                    Token::GreaterThanOrEqual
                }
                _ => Token::GreaterThan,
            },
            '<' => match self.peek_char() {
                '=' => {
                    self.read_char();
                    Token::LessThanOrEqual
                }
                _ => Token::LessThan,
            },
            '+' => Token::Add,
            '-' => Token::Minus,
            '*' => Token::Asterisk,
            '/' => Token::Slash,
            '^' => Token::Caret,
            '%' => Token::Percent,
            '!' => match self.peek_char() {
                '=' => {
                    self.read_char();
                    Token::NotEqual
                }
                _ => Token::Exclamation,
            },
            '?' => Token::Question,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            STOP_CHAR => Token::Eof,
            // 这里将 ' 和 " 混淆在一起
            // 例如 'xxx" 是可以的,
            // TODO: fix it
            '\'' => Token::String(self.read_string()),
            '\"' => Token::String(self.read_string()),
            ch => {
                if is_letter(ch) {
                    let ident_str = self.read_identifier();
                    let keyword = find_keyword(&ident_str);
                    let t = match keyword {
                        Keyword::UserIdent => Token::Ident(ident_str),
                        _ => Token::KeyWord(keyword),
                    };

                    return t;
                } else if is_digit(self.cur_read_char) {
                    return Token::Number(self.read_number());
                }

                Token::KeyWord(Keyword::UserIdent)
            }
        };
        self.read_char();

        t
    }

    fn read_char(&mut self) {
        if self.read_pos >= self.origin_str.len() {
            self.cur_read_char = STOP_CHAR;
        } else {
            self.cur_read_char = self.origin_str.as_bytes()[self.read_pos] as char;
        }

        self.pos = self.read_pos;
        self.read_pos += 1;
    }

    fn skip_space(&mut self) {
        loop {
            if self.cur_read_char == ' '
                || self.cur_read_char == '\t'
                || self.cur_read_char == '\n'
                || self.cur_read_char == '\r'
            {
                self.read_char();
                continue;
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> char {
        if self.read_pos >= self.origin_str.len() {
            STOP_CHAR
        } else {
            self.origin_str.as_bytes()[self.read_pos] as char
        }
    }

    fn read_string(&mut self) -> String {
        let pre_pos = self.pos + 1;
        loop {
            self.read_char();

            if self.cur_read_char == '\''
                || self.cur_read_char == '\"'
                || self.cur_read_char == STOP_CHAR
            {
                break;
            }
        }

        String::from(&self.origin_str[pre_pos..self.pos])
    }

    fn read_identifier(&mut self) -> String {
        let pre_pos = self.pos;

        loop {
            if is_letter(self.cur_read_char) || is_digit(self.cur_read_char) {
                self.read_char();
            } else {
                break;
            }
        }

        String::from(&self.origin_str[pre_pos..self.pos])
    }

    fn read_number(&mut self) -> String {
        let pre_pos = self.pos;
        loop {
            if is_digit(self.cur_read_char) || self.cur_read_char == '.' {
                self.read_char()
            } else {
                break;
            }
        }

        String::from(&self.origin_str[pre_pos..self.pos])
    }
}

// 是否是 字母开头
fn is_letter(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

// 是否是 数字
fn is_digit(ch: char) -> bool {
    ch.is_numeric()
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    pub fn basic_test_1() {
        let mut sql = "SELECT * FROM TABLE_NAME_1;";

        let mut lexer = Lexer::new_lexer(sql.to_string());

        let mut result = vec![
            Token::KeyWord(Keyword::Select),
            Token::Asterisk,
            Token::KeyWord(Keyword::From),
            Token::Ident("TABLE_NAME_1".to_string()),
            Token::Semicolon,
            Token::Eof,
        ];

        for t in result {
            let token = lexer.next_token();
            assert_eq!(t, token);
        }

        sql = "CREATE Table movie (
               id Integer primary key,
               title string not null,
               release_year integer index,
               imdb_id string index unique,
               bluray boolean not null default true
                );";

        result = vec![
            Token::KeyWord(Keyword::Create),
            Token::KeyWord(Keyword::Table),
            Token::Ident("movie".to_string()),
            Token::LeftParen,
            Token::Ident("id".to_string()),
            Token::KeyWord(Keyword::Integer),
            Token::KeyWord(Keyword::Primary),
            Token::KeyWord(Keyword::Key),
            Token::Comma,
            Token::Ident("title".to_string()),
            Token::KeyWord(Keyword::String),
            Token::KeyWord(Keyword::Not),
            Token::KeyWord(Keyword::Null),
            Token::Comma,
            Token::Ident("release_year".to_string()),
            Token::KeyWord(Keyword::Integer),
            Token::KeyWord(Keyword::Index),
            Token::Comma,
            Token::Ident("imdb_id".to_string()),
            Token::KeyWord(Keyword::String),
            Token::KeyWord(Keyword::Index),
            Token::KeyWord(Keyword::Unique),
            Token::Comma,
            Token::Ident("bluray".to_string()),
            Token::KeyWord(Keyword::Boolean),
            Token::KeyWord(Keyword::Not),
            Token::KeyWord(Keyword::Null),
            Token::KeyWord(Keyword::Default),
            Token::KeyWord(Keyword::True),
            Token::RightParen,
            Token::Semicolon,
            Token::Eof,
        ];

        lexer.update(sql.to_owned());
        for t in result {
            assert_eq!(lexer.next_token(), t);
        }

        sql = r#"INSERT INTO movie 
             (id, title, release_year) 
             VALUES
             (1, "Sicario", 2015),
             (2, "Stalker", 1979),
             (3, "Her", 2013);"#;

        result = vec![
            Token::KeyWord(Keyword::Insert),
            Token::KeyWord(Keyword::Into),
            Token::Ident("movie".to_owned()),
            Token::LeftParen,
            Token::Ident("id".to_owned()),
            Token::Comma,
            Token::Ident("title".to_owned()),
            Token::Comma,
            Token::Ident("release_year".to_owned()),
            Token::RightParen,
            Token::KeyWord(Keyword::Values),
            Token::LeftParen,
            Token::Number("1".to_string()),
            Token::Comma,
            Token::String("Sicario".to_string()),
            Token::Comma,
            Token::Number("2015".to_string()),
            Token::RightParen,
            Token::Comma,
            Token::LeftParen,
            Token::Number("2".to_string()),
            Token::Comma,
            Token::String("Stalker".to_string()),
            Token::Comma,
            Token::Number("1979".to_string()),
            Token::RightParen,
            Token::Comma,
            Token::LeftParen,
            Token::Number("3".to_string()),
            Token::Comma,
            Token::String("Her".to_string()),
            Token::Comma,
            Token::Number("2013".to_string()),
            Token::RightParen,
            Token::Semicolon,
            Token::Eof,
        ];

        lexer.update(sql.to_owned());
        for t in result {
            assert_eq!(lexer.next_token(), t);
        }

        sql = r#"INSERT INTO movies VALUES 
                (1,  'Stalker', 1, 1, 1979, 8.2), 
                (2,  'Sicario', 2, 2, 2015, 7.6), 
                (12, 'Eternal Sunshine of the Spotless Mind', 5, 3, 2004, 8.3);"#;

        result = vec![
            Token::KeyWord(Keyword::Insert),
            Token::KeyWord(Keyword::Into),
            Token::Ident("movies".to_string()),
            Token::KeyWord(Keyword::Values),
            Token::LeftParen,
            Token::Number("1".to_string()),
            Token::Comma,
            Token::String("Stalker".to_string()),
            Token::Comma,
            Token::Number("1".to_string()),
            Token::Comma,
            Token::Number("1".to_string()),
            Token::Comma,
            Token::Number("1979".to_string()),
            Token::Comma,
            Token::Number("8.2".to_string()),
            Token::RightParen,
            Token::Comma,
            Token::LeftParen,
            Token::Number("2".to_string()),
            Token::Comma,
            Token::String("Sicario".to_string()),
            Token::Comma,
            Token::Number("2".to_string()),
            Token::Comma,
            Token::Number("2".to_string()),
            Token::Comma,
            Token::Number("2015".to_string()),
            Token::Comma,
            Token::Number("7.6".to_string()),
            Token::RightParen,
            Token::Comma,
            Token::LeftParen,
            Token::Number("12".to_string()),
            Token::Comma,
            Token::String("Eternal Sunshine of the Spotless Mind".to_string()),
            Token::Comma,
            Token::Number("5".to_string()),
            Token::Comma,
            Token::Number("3".to_string()),
            Token::Comma,
            Token::Number("2004".to_string()),
            Token::Comma,
            Token::Number("8.3".to_string()),
            Token::RightParen,
            Token::Semicolon,
            Token::Eof,
        ];

        lexer.update(sql.to_owned());
        for t in result {
            assert_eq!(lexer.next_token(), t);
        }
    }

    #[test]
    pub fn basic_test_2() {
        let mut sql = "DROP TABLE TABLE_NAME_1;";

        let mut lexer = Lexer::new_lexer(sql.to_string());

        let mut result = vec![
            Token::KeyWord(Keyword::Drop),
            Token::KeyWord(Keyword::Table),
            Token::Ident("TABLE_NAME_1".to_string()),
            Token::Semicolon,
            Token::Eof,
        ];

        for t in result {
            assert_eq!(lexer.next_token(), t);
        }

        sql = "DELETE FROM studios WHERE id = 1;";

        lexer.update(sql.to_string());

        result = vec![
            Token::KeyWord(Keyword::Delete),
            Token::KeyWord(Keyword::From),
            Token::Ident("studios".to_string()),
            Token::KeyWord(Keyword::Where),
            Token::Ident("id".to_string()),
            Token::Equal,
            Token::Number("1".to_string()),
            Token::Semicolon,
            Token::Eof,
        ];

        for t in result {
            assert_eq!(t, lexer.next_token());
        }

        sql = "UPDATE movies set id = 1;";

        lexer.update(sql.to_string());
        result = vec![
            Token::KeyWord(Keyword::Update),
            Token::Ident("movies".to_string()),
            Token::KeyWord(Keyword::Set),
            Token::Ident("id".to_string()),
            Token::Equal,
            Token::Number("1".to_owned()),
            Token::Semicolon,
            Token::Eof,
        ];

        for t in result {
            assert_eq!(t, lexer.next_token());
        }

        sql = "Select 3.14 * 8.091;";
        lexer.update(sql.to_string());

        result = vec![
            Token::KeyWord(Keyword::Select),
            Token::Number("3.14".to_string()),
            Token::Asterisk,
            Token::Number("8.091".to_string()),
            Token::Semicolon,
            Token::Eof,
        ];

        for t in result {
            assert_eq!(t, lexer.next_token());
        }

        sql = "select 1 ^ 8 / infinity, 8 ^ 10, infinity, infinity / infinity;";
        lexer.update(sql.to_string());

        result = vec![
            Token::KeyWord(Keyword::Select),
            Token::Number("1".to_string()),
            Token::Caret,
            Token::Number("8".to_string()),
            Token::Slash,
            Token::KeyWord(Keyword::Infinity),
            Token::Comma,
            Token::Number("8".to_string()),
            Token::Caret,
            Token::Number("10".to_string()),
            Token::Comma,
            Token::KeyWord(Keyword::Infinity),
            Token::Comma,
            Token::KeyWord(Keyword::Infinity),
            Token::Slash,
            Token::KeyWord(Keyword::Infinity),
            Token::Semicolon,
            Token::Eof,
        ];

        for t in result {
            assert_eq!(t, lexer.next_token());
        }

        sql = "SELECT Not True, Not False, Not Null;";

        result = vec![
            Token::KeyWord(Keyword::Select),
            Token::KeyWord(Keyword::Not),
            Token::KeyWord(Keyword::True),
            Token::Comma,
            Token::KeyWord(Keyword::Not),
            Token::KeyWord(Keyword::False),
            Token::Comma,
            Token::KeyWord(Keyword::Not),
            Token::KeyWord(Keyword::Null),
            Token::Semicolon,
            Token::Eof,
        ];

        lexer.update(sql.to_owned());

        for t in result {
            assert_eq!(t, lexer.next_token());
        }

        sql = r#"SELECT m.id, m.title, g.name AS genre, m.released, s.name
                 AS studio
                 FROM movies m JOIN genres g ON m.genre_id = g.id,
                      studio s JOIN movies good ON good.studio_id = s.id
                 AND good.rating >= 8
                 WHERE m.studio_id = s.id AND m.released >= 2000 AND g.id = 1
                 ORDER BY m.title ASC;"#;
        result = vec![
            Token::KeyWord(Keyword::Select),
            Token::Ident("m".to_owned()),
            Token::Period,
            Token::Ident("id".to_owned()),
            Token::Comma,
            Token::Ident("m".to_owned()),
            Token::Period,
            Token::Ident("title".to_owned()),
            Token::Comma,
            Token::Ident("g".to_owned()),
            Token::Period,
            Token::Ident("name".to_owned()),
            Token::KeyWord(Keyword::As),
            Token::Ident("genre".to_owned()),
            Token::Comma,
            Token::Ident("m".to_string()),
            Token::Period,
            Token::Ident("released".to_string()),
            Token::Comma,
            Token::Ident("s".to_string()),
            Token::Period,
            Token::Ident("name".to_string()),
            Token::KeyWord(Keyword::As),
            Token::Ident("studio".to_owned()),
            Token::KeyWord(Keyword::From),
            Token::Ident("movies".to_string()),
            Token::Ident("m".to_string()),
            Token::KeyWord(Keyword::Join),
            Token::Ident("genres".to_owned()),
            Token::Ident("g".to_owned()),
            Token::KeyWord(Keyword::On),
            Token::Ident("m".to_owned()),
            Token::Period,
            Token::Ident("genre_id".to_owned()),
            Token::Equal,
            Token::Ident("g".to_owned()),
            Token::Period,
            Token::Ident("id".to_owned()),
            Token::Comma,
            Token::Ident("studio".to_owned()),
            Token::Ident("s".to_owned()),
            Token::KeyWord(Keyword::Join),
            Token::Ident("movies".to_owned()),
            Token::Ident("good".to_owned()),
            Token::KeyWord(Keyword::On),
            Token::Ident("good".to_owned()),
            Token::Period,
            Token::Ident("studio_id".to_owned()),
            Token::Equal,
            Token::Ident("s".to_owned()),
            Token::Period,
            Token::Ident("id".to_owned()),
            Token::KeyWord(Keyword::And),
            Token::Ident("good".to_owned()),
            Token::Period,
            Token::Ident("rating".to_owned()),
            Token::GreaterThanOrEqual,
            Token::Number("8".to_owned()),
            Token::KeyWord(Keyword::Where),
            Token::Ident("m".to_owned()),
            Token::Period,
            Token::Ident("studio_id".to_owned()),
            Token::Equal,
            Token::Ident("s".to_owned()),
            Token::Period,
            Token::Ident("id".to_owned()),
            Token::KeyWord(Keyword::And),
            Token::Ident("m".to_owned()),
            Token::Period,
            Token::Ident("released".to_owned()),
            Token::GreaterThanOrEqual,
            Token::Number("2000".to_owned()),
            Token::KeyWord(Keyword::And),
            Token::Ident("g".to_owned()),
            Token::Period,
            Token::Ident("id".to_owned()),
            Token::Equal,
            Token::Number("1".to_owned()),
            Token::KeyWord(Keyword::Order),
            Token::KeyWord(Keyword::By),
            Token::Ident("m".to_owned()),
            Token::Period,
            Token::Ident("title".to_owned()),
            Token::KeyWord(Keyword::Asc),
            Token::Semicolon,
            Token::Eof,
        ];

        lexer.update(sql.to_owned());

        for t in result {
            assert_eq!(t, lexer.next_token());
        }
    }
}
