use std::{fs, process::exit};

mod environment;
mod interpreter;
mod parser;
mod scanner;
mod token;

use interpreter::*;
use parser::*;
use scanner::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} [script]", args[0]);
        exit(1);
    }

    if let Err(err) = run_file(&args[1]) {
        eprintln!("{err}");
        std::process::exit(70);
    }
}

fn run_file(path: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let mut scanner = Scanner::new(text.as_str());
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);
    let stmts = parser.parse()?;

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&stmts)?;

    // print!("{expr:#?}");

    // for token in tokens.iter() {
    //     println!("{:?}", token);
    // }
    Ok(())
}
