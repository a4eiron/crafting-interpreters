use super::{LoxClass, RuntimeError, RuntimeResult, Value};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::lexer::Token;

#[derive(Debug, Clone)]
pub struct LoxInstance {
    class: Rc<LoxClass>,
    fields: Rc<RefCell<HashMap<String, Value>>>,
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self {
            class,
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Value> {
        if let Some(value) = self.fields.borrow().get(name.lexeme()) {
            return Ok(value.clone());
        }
        if let Some(method) = self.class.find_method(name.lexeme()) {
            let bound = method.bind(self.clone())?;
            return Ok(Value::Callable(Rc::new(bound)));
        }

        Err(RuntimeError {
            token: Some(name.clone()),
            message: format!("undefined property {}", name.lexeme()),
        })
    }
    pub fn set(&self, name: Token, value: Value) -> RuntimeResult<()> {
        self.fields.borrow_mut().insert(name.lexeme().into(), value);
        Ok(())
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "instance of {}", self.class)
    }
}
