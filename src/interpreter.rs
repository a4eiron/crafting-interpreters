use crate::environment::Environment;
use crate::parser::{Expr, Stmt};
use crate::token::{Literal, Token, TokenType};

use std::{cell::RefCell, fmt, rc::Rc};

pub type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl std::error::Error for RuntimeError {}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line: {} | {}", self.token.line(), self.message)
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
            Self::Nil => write!(f, "<nil>"),
            Self::Bool(b) => write!(f, "{}", b),
        }
    }
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute_block(&mut self, stmts: &[Stmt], env: Environment) -> Result<()> {
        let previous = std::mem::replace(&mut self.environment, Rc::new(RefCell::new(env)));
        for stmt in stmts {
            self.execute(stmt)?;
        }
        self.environment = previous;
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Var { name, initializer } => {
                let value = self.evaluate(initializer)?;
                self.environment
                    .borrow_mut()
                    .define(name.lexeme().to_string(), value);
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{value}");
            }
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
            }
            Stmt::Block(stmts) => {
                self.execute_block(
                    stmts,
                    Environment::new_with_env(Rc::clone(&self.environment)),
                )?;
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Var(t) => self.environment.borrow().get(t),
            Expr::Grouping(g) => self.evaluate(g),
            Expr::Literal(l) => literal(l),
            Expr::Unary { operator, right } => {
                let value = self.evaluate(right)?;
                unary(operator, value)
            }
            Expr::Binary {
                left,
                right,
                operator,
            } => {
                let v_left = self.evaluate(left)?;
                let v_right = self.evaluate(right)?;
                binary(operator, v_left, v_right)
            }
            Expr::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                let v_condition = self.evaluate(condition)?;
                if is_truthy(&v_condition) {
                    self.evaluate(then_branch)
                } else {
                    self.evaluate(else_branch)
                }
            }
            Expr::Assignment { name, value } => {
                let v = self.evaluate(value)?;
                self.environment.borrow_mut().assign(name, v.clone())?;
                Ok(v)
            }
        }
    }
}

fn literal(literal: &Literal) -> Result<Value> {
    match literal {
        Literal::Number(n) => Ok(Value::Number(*n)),
        Literal::String(s) => Ok(Value::String(s.clone())),
        Literal::True => Ok(Value::Bool(true)),
        Literal::False => Ok(Value::Bool(false)),
        Literal::Nil => Ok(Value::Nil),
    }
}

fn unary(operator: &Token, value: Value) -> Result<Value> {
    match operator.token_type() {
        TokenType::Minus => match value {
            Value::Number(n) => Ok(Value::Number(-n)),
            _ => Err(RuntimeError {
                token: operator.clone(),
                message: String::from("operand must be a number"),
            }),
        },
        TokenType::Bang => Ok(Value::Bool(!is_truthy(&value))),
        _ => unreachable!(),
    }
}

fn binary(operator: &Token, left: Value, right: Value) -> Result<Value> {
    let err = |msg: &str| {
        Err(RuntimeError {
            token: operator.clone(),
            message: String::from(msg),
        })
    };
    match (operator.token_type(), left, right) {
        (TokenType::Minus, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
        (TokenType::Star, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
        (TokenType::Slash, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
        (TokenType::Greater, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
        (TokenType::GreaterEqual, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
        (TokenType::Lesser, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
        (TokenType::LesserEqual, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
        (TokenType::Plus, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
        (TokenType::Plus, Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
        (TokenType::Plus, _, _) => err("operands must be numbers or strings"),

        (TokenType::EqualEqual, ref a, Value::Bool(b)) => Ok(Value::Bool(is_truthy(a) == b)),
        (TokenType::EqualEqual, Value::Bool(a), ref b) => Ok(Value::Bool(a == is_truthy(b))),
        (TokenType::EqualEqual, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a == b)),
        (TokenType::EqualEqual, Value::String(a), Value::String(b)) => Ok(Value::Bool(a == b)),
        // (TokenType::EqualEqual, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
        (TokenType::EqualEqual, _, _) => Ok(Value::Bool(false)),

        (TokenType::BangEqual, ref a, Value::Bool(b)) => Ok(Value::Bool(is_truthy(a) != b)),
        (TokenType::BangEqual, Value::Bool(a), ref b) => Ok(Value::Bool(a != is_truthy(b))),
        (TokenType::BangEqual, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a != b)),
        (TokenType::BangEqual, Value::String(a), Value::String(b)) => Ok(Value::Bool(a != b)),
        // (TokenType::BangEqual, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),
        (
            TokenType::Minus
            | TokenType::Star
            | TokenType::Slash
            | TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Lesser
            | TokenType::LesserEqual,
            _,
            _,
        ) => err("operands must be numbers"),
        _ => err("unknown operator"),
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(b) => *b,
        Value::String(s) => !s.is_empty(),
        Value::Number(n) => *n != 0.0,
    }
}
