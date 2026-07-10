use crate::lexer::Token;
use crate::parser::FuncStmt;
use crate::runtime::*;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

pub trait Callable: fmt::Display {
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> RuntimeResult<Value>;
    fn arity(&self) -> usize;
}

impl fmt::Debug for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Callable {}>", self)
    }
}
