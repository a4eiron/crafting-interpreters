use crate::runtime::*;

use std::fmt;

pub trait Callable: fmt::Display {
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> RuntimeResult<Value>;
    fn arity(&self) -> usize;
}

impl fmt::Debug for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Callable {}>", self)
    }
}
