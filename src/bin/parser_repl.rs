use std::io;
use std::io::Write;

use shaun::parser::Parser;

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
    let mut p = Parser::new_parser("".to_owned());
    let mut input = String::new();
    loop {
        input.clear();
        std::print!("{green}parser> {default}");
        std::io::stdout().flush().unwrap();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input.trim() == "quit" {
                    break;
                }
                p.update(&input);
                match p.parse_stmt() {
                    Ok(s) => {
                        println!("{:?}", s);
                    }
                    Err(e) => {
                        println!("ParseErr: {:?}", e);
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
