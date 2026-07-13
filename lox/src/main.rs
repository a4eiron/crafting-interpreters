use std::{fs, process::exit};

use lox::Interpreter;
use lox::LoxError;
use lox::Parser;
use lox::Resolver;
use lox::Scanner;

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

fn run_file(path: &str) -> std::result::Result<(), LoxError> {
    let text = fs::read_to_string(path)?;
    let mut scanner = Scanner::new(text.as_str());
    let tokens = scanner.scan_tokens()?;

    // for token in tokens.iter() {
    //     println!("{:?}", token);
    // }
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse()?;

    let mut interpreter = Interpreter::new();
    let mut resolver = Resolver::new(&mut interpreter);
    resolver.resolve(&stmts)?;

    interpreter.interpret(&stmts)?;

    Ok(())
}
