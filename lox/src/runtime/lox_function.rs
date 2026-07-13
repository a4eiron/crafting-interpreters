use super::{Callable, ControlFlow, Environment, LoxInstance, RuntimeResult, Value};
use crate::parser::FuncStmt;

use std::fmt;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct LoxFunction {
    is_initializer: bool,
    declaration: Rc<FuncStmt>,
    closure: Rc<RefCell<Environment>>,
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<func {}>", self.declaration.name.lexeme())
    }
}

impl LoxFunction {
    pub fn new(
        declaration: Rc<FuncStmt>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            is_initializer,
            declaration,
            closure,
        }
    }

    pub fn bind(&self, instance: LoxInstance) -> RuntimeResult<Self> {
        let env = Environment::new_with_env(self.closure.clone());
        env.borrow_mut()
            .define_str("this", Value::Instance(instance))?;

        Ok(Self {
            is_initializer: self.is_initializer,
            declaration: self.declaration.clone(),
            closure: env,
        })
    }

    pub fn is_getter(&self) -> bool {
        self.declaration.getter
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
    fn call(&self, interpreter: &mut super::Interpreter, args: Vec<Value>) -> RuntimeResult<Value> {
        let env = Environment::new_with_env(Rc::clone(&self.closure));
        for (token, value) in self.declaration.params.iter().zip(args.into_iter()) {
            env.borrow_mut().define(token, value)?;
        }
        let value = match interpreter.execute_block(&self.declaration.body, env) {
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
