use std::fmt;
use std::{cell::RefCell, rc::Rc};

use crate::lexer::Token;
use crate::runtime::{Callable, ControlFlow, LoxInstance, RuntimeResult, Value};
use crate::{parser::FuncStmt, runtime::Environment};

#[derive(Debug, Clone)]
pub struct LoxFunction {
    declaration: FuncStmt,
    closure: Rc<RefCell<Environment>>,
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<func {}>", self.declaration.name.lexeme())
    }
}

impl LoxFunction {
    pub fn new(declaration: FuncStmt, closure: Rc<RefCell<Environment>>) -> Self {
        Self {
            declaration,
            closure,
        }
    }

    pub fn bind(&self, instance: LoxInstance) -> RuntimeResult<Self> {
        let mut env = Environment::new_with_env(self.closure.clone());
        env.define(
            &Token::new(crate::lexer::TokenType::This, 0, "this".to_string(), None),
            Value::Instance(instance),
        )?;

        Ok(Self {
            declaration: self.declaration.clone(),
            closure: Rc::new(RefCell::new(env)),
        })
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
    fn call(&self, interpreter: &mut super::Interpreter, args: Vec<Value>) -> RuntimeResult<Value> {
        let mut env = Environment::new_with_env(Rc::clone(&self.closure));
        for (token, value) in self.declaration.params.iter().zip(args.into_iter()) {
            env.define(token, value)?;
        }
        let value = match interpreter.execute_block(&self.declaration.body, env) {
            Err(e) => match e {
                ControlFlow::Break => Value::Nil,
                ControlFlow::Return(v) => v,
                ControlFlow::Error(err) => return Err(err),
            },
            Ok(_) => Value::Nil,
        };
        Ok(value)
    }
}
