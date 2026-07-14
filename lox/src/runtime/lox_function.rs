use super::{Callable, ControlFlow, Environment, LoxInstance, RuntimeResult, Value};
use crate::lexer::Token;
use crate::parser::Stmt;

use std::fmt;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct LoxFunction {
    name: Option<String>,
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
    pub is_getter: bool,
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(n) => write!(f, "<func {}>", n),
            None => write!(f, "<fn>"),
        }
    }
}

impl LoxFunction {
    pub fn new(
        name: Option<String>,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
        is_getter: bool,
    ) -> Self {
        Self {
            name,
            params,
            body,
            closure,
            is_initializer,
            is_getter,
        }
    }

    pub fn is_getter(&self) -> bool {
        self.is_getter
    }

    pub fn bind(&self, instance: LoxInstance) -> RuntimeResult<Self> {
        let env = Environment::new_with_env(self.closure.clone());
        env.borrow_mut()
            .define_str("this", Value::Instance(instance))?;

        Ok(Self {
            name: self.name.clone(),
            params: self.params.clone(),
            body: self.body.clone(),
            is_initializer: self.is_initializer,
            is_getter: self.is_getter,
            closure: env,
        })
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }
    fn call(&self, interpreter: &mut super::Interpreter, args: Vec<Value>) -> RuntimeResult<Value> {
        let env = Environment::new_with_env(Rc::clone(&self.closure));
        for (token, value) in self.params.iter().zip(args) {
            env.borrow_mut().define(token, value)?;
        }
        let value = match interpreter.execute_block(&self.body, env) {
            Err(e) => match e {
                ControlFlow::Continue => Value::Nil,
                ControlFlow::Break => Value::Nil,
                ControlFlow::Return(v) => v,
                ControlFlow::Error(err) => return Err(err),
            },
            Ok(_) => {
                if self.is_initializer {
                    Environment::get_at_str(self.closure.clone(), 0, "this")?
                } else {
                    Value::Nil
                }
            }
        };
        Ok(value)
    }
}
