use std::{fs, process::exit};

mod interpreter;
mod parser;
mod scanner;
mod token;

use interpreter::*;
use parser::*;
use scanner::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} [script]", args[0]);
        exit(1);
    }

    run_file(&args[1])?;

    Ok(())
}

fn run_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let mut scanner = Scanner::new(text.as_str());
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;

    let interpreter = Interpreter::new();
    let val = interpreter.interpret(&expr)?;
    println!("{val}");

    // print!("{expr:#?}");

    // for token in tokens.iter() {
    //     println!("{:?}", token);
    // }
    Ok(())
}
