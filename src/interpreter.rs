use crate::environment::Environment;
use crate::parser::{Expr, FuncStmt, ParseError, ParseResult, Stmt};
use crate::token::{Literal, Token, TokenType};

use std::{cell::RefCell, fmt, rc::Rc};

///////////////////////////////////////////////////////////////////////////////////////
pub type RuntimeResult<T> = std::result::Result<T, RuntimeError>;

pub enum ControlFlow {
    Error(RuntimeError),
    Return(Value),
}

impl From<RuntimeError> for ControlFlow {
    fn from(err: RuntimeError) -> Self {
        ControlFlow::Error(err)
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Option<Token>,
    pub message: String,
}

impl std::error::Error for RuntimeError {}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.token {
            Some(token) => write!(f, "line: {} | {}", token.line(), self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl RuntimeError {
    fn new(msg: &str) -> Self {
        Self {
            token: None,
            message: msg.to_string(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Callable(Rc<dyn Callable>),
}

impl Value {
    fn add(self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            (_, _) => Err(RuntimeError::new("operands must be numbers or strings")),
        }
    }

    fn sub(self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            (_, _) => Err(RuntimeError::new("operands must be numbers")),
        }
    }

    fn mul(self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            (_, _) => Err(RuntimeError::new("operands must be numbers")),
        }
    }

    fn divide(self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
            (_, _) => Err(RuntimeError::new("operands must be numbers")),
        }
    }

    fn equal(&self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a == b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a == b)),
            (ref a, Value::Bool(b)) => Ok(Value::Bool(is_truthy(a) == *b)),
            (Value::Bool(a), ref b) => Ok(Value::Bool(*a == is_truthy(b))),
            (_, _) => Err(RuntimeError::new("incompatable")),
        }
    }

    fn not_equal(&self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a != b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a != b)),
            (ref a, Value::Bool(b)) => Ok(Value::Bool(is_truthy(a) != *b)),
            (Value::Bool(a), ref b) => Ok(Value::Bool(*a != is_truthy(b))),
            (_, _) => Err(RuntimeError::new("incompatable")),
        }
    }

    fn greater(&self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
            (_, _) => Err(RuntimeError::new("incompatable")),
        }
    }

    fn greater_equal(&self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
            (_, _) => Err(RuntimeError::new("incompatable")),
        }
    }

    fn lesser(&self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
            (_, _) => Err(RuntimeError::new("incompatable")),
        }
    }

    fn lesser_equal(&self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
            (_, _) => Err(RuntimeError::new("incompatable")),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Nil => write!(f, "<nil>"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Callable(func) => write!(f, "{:?}", func),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////
pub trait Callable {
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> RuntimeResult<Value>;
    fn arity(&self) -> usize;
}

impl fmt::Debug for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<func>")
    }
}
////////////////////////////////////////////////////////////////////////////////////
pub struct DamnFunc {
    declaration: FuncStmt,
}

impl DamnFunc {
    fn new(declaration: Stmt) -> ParseResult<Self> {
        match declaration {
            Stmt::Func(stmt) => Ok(Self { declaration: stmt }),
            _ => Err(ParseError::new(0, "")),
        }
    }
}

impl Callable for DamnFunc {
    fn arity(&self) -> usize {
        self.declaration.args.len()
    }
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> RuntimeResult<Value> {
        let mut env = Environment::new_with_env(Rc::clone(&interpreter.environment));
        for (token, value) in self.declaration.args.iter().zip(args.into_iter()) {
            env.define(token, value)?;
        }
        let value = match interpreter.execute_block(&self.declaration.body, env) {
            Err(e) => match e {
                ControlFlow::Return(v) => v,
                _ => Value::Nil,
            },
            Ok(_) => Value::Nil,
        };
        return Ok(value);
    }
}

////////////////////////////////////////////////////////////////////////////////////
pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let global_env = Environment::new();

        struct Clock;

        impl Callable for Clock {
            fn arity(&self) -> usize {
                0
            }
            fn call(
                &self,
                _interpreter: &mut Interpreter,
                _args: Vec<Value>,
            ) -> RuntimeResult<Value> {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                Ok(Value::Number(now))
            }
        }

        global_env
            .borrow_mut()
            .define(
                &Token::new(
                    TokenType::Identifier,
                    0,
                    String::from("clock"),
                    Some(Literal::Nil),
                ),
                Value::Callable(Rc::new(Clock)),
            )
            .unwrap();

        Self {
            globals: Rc::clone(&global_env),
            environment: Rc::clone(&global_env),
        }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> RuntimeResult<()> {
        for stmt in stmts {
            if let Err(flow) = self.execute(stmt) {
                match flow {
                    ControlFlow::Error(e) => return Err(e),
                    ControlFlow::Return(_) => {
                        return Err(RuntimeError {
                            token: Some(Token::new(
                                TokenType::Return,
                                0,
                                String::from("return"),
                                None,
                            )),
                            message: String::from("cannot return from top-level code."),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn execute_block(
        &mut self,
        stmts: &[Stmt],
        env: Environment,
    ) -> std::result::Result<(), ControlFlow> {
        let previous = std::mem::replace(&mut self.environment, Rc::new(RefCell::new(env)));
        for stmt in stmts {
            if let Err(e) = self.execute(stmt) {
                self.environment = previous;
                return Err(e);
            }
        }
        self.environment = previous;
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> std::result::Result<(), ControlFlow> {
        match stmt {
            Stmt::Var(stmt) => {
                let value = self.evaluate(&stmt.initializer)?;
                self.environment.borrow_mut().define(&stmt.name, value)?;
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
            Stmt::If(stmt) => {
                let value = self.evaluate(&stmt.condition)?;
                if is_truthy(&value) {
                    self.execute(&stmt.then_branch)?;
                } else if let Some(stmt) = &stmt.else_branch {
                    self.execute(&stmt)?
                }
            }
            Stmt::While(stmt) => {
                let mut value = self.evaluate(&stmt.condition)?;
                while is_truthy(&value) {
                    self.execute(&stmt.body)?;
                    value = self.evaluate(&stmt.condition)?;
                }
            }
            Stmt::Func(stmt) => {
                let func = DamnFunc::new(Stmt::Func(FuncStmt {
                    name: stmt.name.clone(),
                    args: stmt.args.clone(),
                    body: stmt.body.clone(),
                }))
                .unwrap();
                self.environment
                    .borrow_mut()
                    .define(&stmt.name, Value::Callable(Rc::new(func)))?;
            }
            Stmt::Return(stmt) => {
                let mut v = Value::Nil;
                if !matches!(&stmt.value, Expr::Literal(Literal::Nil)) {
                    v = self.evaluate(&stmt.value)?;
                }
                return Err(ControlFlow::Return(v));
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> RuntimeResult<Value> {
        match expr {
            Expr::Literal(l) => literal(l),
            Expr::Grouping(g) => self.evaluate(g),
            Expr::Var(t) => self.environment.borrow().get(t),
            Expr::Unary(expr) => {
                let value = self.evaluate(&expr.right)?;
                unary(&expr.operator, value)
            }
            Expr::Binary(expr) => {
                let v_left = self.evaluate(&expr.left)?;
                let v_right = self.evaluate(&expr.right)?;
                binary(&expr.operator, v_left, &v_right)
            }
            Expr::Logical(expr) => {
                let v_left = self.evaluate(&expr.left)?;
                if &expr.operator.token_type() == &TokenType::Or {
                    if is_truthy(&v_left) {
                        return Ok(v_left);
                    }
                } else {
                    if !is_truthy(&v_left) {
                        return Ok(v_left);
                    }
                }
                self.evaluate(&expr.right)
            }
            Expr::Conditional(expr) => {
                let v_condition = self.evaluate(&expr.condition)?;
                if is_truthy(&v_condition) {
                    self.evaluate(&expr.then_branch)
                } else {
                    self.evaluate(&expr.else_branch)
                }
            }
            Expr::Assignment(expr) => {
                let v = self.evaluate(&expr.value)?;
                self.environment
                    .borrow_mut()
                    .assign(&expr.name, v.clone())?;
                Ok(v)
            }
            Expr::Call(expr) => {
                let callee = self.evaluate(&expr.callee)?;
                let mut arguments = Vec::new();

                for arg in &expr.args {
                    arguments.push(self.evaluate(&arg)?);
                }

                match callee {
                    Value::Callable(func) => {
                        if expr.args.len() != func.arity() {
                            return Err(RuntimeError {
                                token: Some(expr.paren.clone()),
                                message: format!(
                                    "expected {} arguments, got  {}",
                                    func.arity(),
                                    expr.args.len()
                                ),
                            });
                        }
                        func.call(self, arguments)
                    }
                    _ => Err(RuntimeError {
                        token: Some(expr.paren.clone()),
                        message: format!("can call only functions"),
                    }),
                }
            }
        }
    }
}

fn literal(literal: &Literal) -> RuntimeResult<Value> {
    match literal {
        Literal::Number(n) => Ok(Value::Number(*n)),
        Literal::String(s) => Ok(Value::String(s.clone())),
        Literal::True => Ok(Value::Bool(true)),
        Literal::False => Ok(Value::Bool(false)),
        Literal::Nil => Ok(Value::Nil),
    }
}

fn unary(operator: &Token, value: Value) -> RuntimeResult<Value> {
    match operator.token_type() {
        TokenType::Minus => match value {
            Value::Number(n) => Ok(Value::Number(-n)),
            _ => Err(RuntimeError {
                token: Some(operator.clone()),
                message: String::from("operand must be a number"),
            }),
        },
        TokenType::Bang => Ok(Value::Bool(!is_truthy(&value))),
        _ => unreachable!(),
    }
}

fn binary(operator: &Token, left: Value, right: &Value) -> RuntimeResult<Value> {
    let err = |msg: &str| {
        Err(RuntimeError {
            token: Some(operator.clone()),
            message: String::from(msg),
        })
    };

    match operator.token_type() {
        TokenType::Plus => left.add(right),
        TokenType::Minus => left.sub(right),
        TokenType::Star => left.mul(right),
        TokenType::Slash => left.divide(right),
        TokenType::EqualEqual => left.equal(right),
        TokenType::BangEqual => left.not_equal(right),
        TokenType::Greater => left.greater(right),
        TokenType::GreaterEqual => left.greater_equal(right),
        TokenType::Lesser => left.lesser(right),
        TokenType::LesserEqual => left.lesser_equal(right),
        _ => err("unknown operator"),
    }
    .map_err(|mut e| {
        e.token = Some(operator.clone());
        e
    })
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(b) => *b,
        Value::String(s) => !s.is_empty(),
        Value::Number(n) => *n != 0.0,
        _ => false,
    }
}
