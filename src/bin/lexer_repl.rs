use std::io;
use std::io::Write;

use shaun::parser::lexer::{self};
use shaun::parser::token::Token;

fn main() {
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

    let green = "\x1b[32m";
    let default = "\x1b[0m";
    let mut l = lexer::Lexer::new_lexer("".to_owned());
    let mut input = String::new();
    loop {
        input.clear();
        print!("{green}lexer>{default} ");
        std::io::stdout().flush().unwrap();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input.trim() == "quit" {
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
