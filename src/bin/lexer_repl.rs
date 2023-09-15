use std::io;

use shaun::parser::lexer::{self};
use shaun::parser::token::Token;

fn main() {
    let green = "\x1b[32m";
    let default = "\x1b[0m";
    let mut l = lexer::Lexer::new_lexer("".to_owned());
    let mut input = String::new();
    loop {
        input.clear();
        print!("{green}lexer: {default} >\n");
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input.as_bytes()[0] as char == 'q'
                    && input.as_bytes()[1] as char == 'u'
                    && input.as_bytes()[2] as char == 'i'
                    && input.as_bytes()[3] as char == 't'
                {
                    break;
                }
                l.update(input.clone());
                loop {
                    let t = l.next_token();
                    match t {
                        Token::Eof => {
                            break;
                        }
                        _ => {
                            println!("{t}");
                        }
                    }
                }
            }
            Err(e) => {
                println!("error: {e}");
                break;
            }
        }
    }
}
