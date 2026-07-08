use crate::parser::FuncStmt;
use crate::runtime::*;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

pub trait Callable {
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> RuntimeResult<Value>;
    fn arity(&self) -> usize;
    fn name(&self) -> String;
}

impl fmt::Debug for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Callable {}>", self.name())
    }
}
////////////////////////////////////////////////////////////////////////////////////
pub struct LoxFunction {
    declaration: FuncStmt,
    closure: Rc<RefCell<Environment>>,
}

impl fmt::Debug for LoxFunction {
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
}

impl Callable for LoxFunction {
    fn name(&self) -> String {
        self.declaration.name.lexeme().to_string()
    }
    fn arity(&self) -> usize {
        self.declaration.args.len()
    }
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> RuntimeResult<Value> {
        let mut env = Environment::new_with_env(Rc::clone(&self.closure));
        for (token, value) in self.declaration.args.iter().zip(args.into_iter()) {
            env.define(token, value)?;
        }
        let value = match interpreter.execute_block(&self.declaration.body, env) {
            Err(e) => match e {
                ControlFlow::Return(v) => v,
                ControlFlow::Error(err) => return Err(err),
            },
            Ok(_) => Value::Nil,
        };
        Ok(value)
    }
}
