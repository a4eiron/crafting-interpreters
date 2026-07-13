use super::{LoxClass, LoxFunction, RuntimeError, RuntimeResult, Value};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::lexer::Token;

pub enum PropResult {
    Value(Value),
    Method(LoxFunction),
    Getter(LoxFunction),
}

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

    pub fn get(&self, name: &Token) -> RuntimeResult<PropResult> {
        if let Some(value) = self.fields.borrow().get(name.lexeme()) {
            return Ok(PropResult::Value(value.clone()));
        }
        if let Some(method) = self.class.find_method(name.lexeme()) {
            if method.is_getter() {
                return Ok(PropResult::Getter(method));
            } else {
                return Ok(PropResult::Method(method));
            }
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
        write!(f, "instance {}", self.class)
    }
}
