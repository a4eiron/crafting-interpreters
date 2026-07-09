use crate::lexer::*;
use crate::parser::*;
use crate::runtime::*;

use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

pub type RuntimeResult<T> = std::result::Result<T, RuntimeError>;

pub enum ControlFlow {
    Error(RuntimeError),
    Return(Value),
}

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    locals: HashMap<ExprId, usize>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let global_env = Environment::new();

        struct Clock;

        impl Callable for Clock {
            fn name(&self) -> String {
                String::from("clock")
            }
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
            locals: HashMap::new(),
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

    pub fn execute_block(
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
                let func = LoxFunction::new(stmt.clone(), Rc::clone(&self.environment));
                self.environment
                    .borrow_mut()
                    .define(&stmt.name, Value::Callable(Rc::new(func)))?;
            }
            Stmt::Return(stmt) => {
                let mut v = Value::Nil;
                if !matches!(&stmt.value.kind, ExprKind::Literal(Literal::Nil)) {
                    v = self.evaluate(&stmt.value)?;
                }
                return Err(ControlFlow::Return(v));
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> RuntimeResult<Value> {
        match &expr.kind {
            ExprKind::Literal(l) => literal(l),
            ExprKind::Grouping(g) => self.evaluate(g),
            ExprKind::Var(t) => {
                if let Some(&distance) = self.locals.get(&expr.id) {
                    Environment::get_at(self.environment.clone(), distance, t)
                } else {
                    self.globals.borrow().get(t)
                }
            }
            ExprKind::Unary(expr) => {
                let value = self.evaluate(&expr.right)?;
                unary(&expr.operator, value)
            }
            ExprKind::Binary(expr) => {
                let v_left = self.evaluate(&expr.left)?;
                let v_right = self.evaluate(&expr.right)?;
                binary(&expr.operator, v_left, &v_right)
            }
            ExprKind::Logical(expr) => {
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
            ExprKind::Conditional(expr) => {
                let v_condition = self.evaluate(&expr.condition)?;
                if is_truthy(&v_condition) {
                    self.evaluate(&expr.then_branch)
                } else {
                    self.evaluate(&expr.else_branch)
                }
            }
            ExprKind::Assignment(e) => {
                let v = self.evaluate(&e.value)?;
                if let Some(&distance) = self.locals.get(&expr.id) {
                    Environment::assign_at(self.environment.clone(), &e.name, v.clone(), distance)?;
                } else {
                    self.globals.borrow_mut().assign(&e.name, v.clone())?;
                }
                Ok(v)
            }
            ExprKind::Call(expr) => {
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

    pub fn resolve(&mut self, expr: &Expr, i: usize) {
        self.locals.insert(expr.id, i);
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

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(b) => *b,
        Value::String(s) => !s.is_empty(),
        Value::Number(n) => *n != 0.0,
        _ => false,
    }
}
