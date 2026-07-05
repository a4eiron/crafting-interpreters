use std::fmt;

use crate::parser::Expr;
use crate::token::{Literal, Token, TokenType};

type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug)]
pub enum RuntimeError {
    InvalidLiteral,
    InvalidOperand,
}

impl std::error::Error for RuntimeError {}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{}", b),
        }
    }
}

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Grouping(g) => self.interpret(g),
            Expr::Literal(l) => literal(l),
            Expr::Unary { operator, right } => {
                let value = self.interpret(right)?;
                unary(operator, value)
            }
            Expr::Binary {
                left,
                right,
                operator,
            } => {
                let v_left = self.interpret(left)?;
                let v_right = self.interpret(right)?;
                binary(operator, v_left, v_right)
            }
            Expr::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                let v_condition = self.interpret(condition)?;
                if is_truthy(&v_condition) {
                    self.interpret(then_branch)
                } else {
                    self.interpret(else_branch)
                }
            }
        }
    }
}

fn literal(literal: &Literal) -> Result<Value> {
    match *literal {
        Literal::Number(n) => Ok(Value::Number(n)),
        Literal::String(ref s) => Ok(Value::String(s.clone())),
        Literal::True => Ok(Value::Bool(true)),
        Literal::False => Ok(Value::Bool(false)),
        Literal::Nil => Ok(Value::Nil),
    }
}

fn unary(operator: &Token, value: Value) -> Result<Value> {
    match (operator.token_type(), value) {
        (TokenType::Bang, Value::Bool(b)) => Ok(Value::Bool(!b)),
        (TokenType::Minus, Value::Number(n)) => Ok(Value::Number(-n)),
        _ => Err(RuntimeError::InvalidLiteral),
    }
}

fn binary(operator: &Token, left: Value, right: Value) -> Result<Value> {
    match (operator.token_type(), left, right) {
        (TokenType::Plus, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
        (TokenType::Minus, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
        (TokenType::Star, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
        (TokenType::Slash, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
        (TokenType::Plus, Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
        (TokenType::Greater, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
        (TokenType::GreaterEqual, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
        (TokenType::Lesser, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
        (TokenType::LesserEqual, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
        (TokenType::BangEqual, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a != b)),
        (TokenType::EqualEqual, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a == b)),
        _ => Err(RuntimeError::InvalidOperand),
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(b) => *b,
        _ => true,
    }
}
