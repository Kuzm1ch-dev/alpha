use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
pub mod tokenizer;
use tokenizer::Tokenizer;
pub mod parser;
use parser::Parser;
pub mod error;
pub mod interpreter;
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        writeln!(io::stderr(), "Usage: {} alpha <filename>", args[0]).unwrap();
        return;
    }
    let filename = &args[1];
    let file_path = PathBuf::from(filename);
    let base_dir = file_path.parent()
        .unwrap_or_else(|| Path::new(""))
        .to_path_buf();

    let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
        String::new()
    });
    if !file_contents.is_empty() {
        let mut tokenizer = Tokenizer::new();
        let input = file_contents.as_str();
        tokenizer.tokenize(input).unwrap();
        if tokenizer.errors.iter().count() > 0 {
            std::process::exit(65);
        }
        let mut parser = Parser::new(tokenizer.get_tokens());
        let exprs = parser.parse();
        match exprs {
            Ok(exprs) => {
                let mut interpreter = interpreter::Interpreter::new_with_base_path(base_dir);
                match interpreter.interpret(exprs) {
                    Ok(_) => {}
                    Err(error) => {
                        eprintln!("{}", error);
                        std::process::exit(70);
                    }
                }
            }
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(65);
            }
        }
    } else {
        println!("Eof  null");
    }
}