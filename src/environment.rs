use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::{Result, RuntimeError, Value};
use crate::token::Token;

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn new_with_env(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }
    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<()> {
        if let Some(variable) = self.values.get_mut(name.lexeme()) {
            *variable = value;
            Ok(())
        } else if let Some(ref mut env) = self.enclosing {
            env.borrow_mut().assign(name, value)
        } else {
            Err(RuntimeError {
                token: name.clone(),
                message: format!("Undefined variable '{}'", name.lexeme()),
            })
        }
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
        if let Some(value) = self.values.get(name.lexeme()) {
            return Ok(value.clone());
        }

        if let Some(ref env) = self.enclosing {
            return env.borrow().get(name);
        }

        Err(RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'", name.lexeme()),
        })
    }
}
