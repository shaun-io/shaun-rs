use std::io::Write;

use shaun::parser::Parser;

const PARSER_HISTORY_NAME: &str = ".shaun_parser_history";

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
        .filter(None, log::LevelFilter::Info)
        .init();

    let green = "\x1b[32m";
    let default = "\x1b[0m";
    let mut p = Parser::new_parser("".to_owned());
    let mut reader = rustyline::DefaultEditor::new().unwrap();
    if reader.load_history(PARSER_HISTORY_NAME).is_err() {
        println!("No previous history.");
    }

    loop {
        match reader.readline(&format!("{green}parser> {default}")) {
            Ok(line) => {
                let _ = reader.add_history_entry(line.as_str());
                if line.trim() == "quit" {
                    break;
                }
                p.update(&line);
                match p.parse_stmt() {
                    Ok(s) => {
                        dbg!(s);
                    }
                    Err(e) => {
                        println!("{:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("error: {e}");
                break;
            }
        }
    }
    reader.save_history(PARSER_HISTORY_NAME).unwrap();
}
