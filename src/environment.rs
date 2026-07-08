use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::interpreter::{RuntimeError, RuntimeResult, Value};
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
    pub fn define(&mut self, name: &Token, value: Value) -> RuntimeResult<()> {
        if self.values.contains_key(name.lexeme()) {
            return Err(RuntimeError {
                token: Some(name.clone()),
                message: format!(
                    "Variable '{}' has already been declared in this scope.",
                    name.lexeme()
                ),
            });
        }

        self.values.insert(name.lexeme().to_string(), value);
        Ok(())
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> RuntimeResult<()> {
        if let Some(variable) = self.values.get_mut(name.lexeme()) {
            *variable = value;
            Ok(())
        } else if let Some(ref mut env) = self.enclosing {
            env.borrow_mut().assign(name, value)
        } else {
            Err(RuntimeError {
                token: Some(name.clone()),
                message: format!("Undefined variable '{}'", name.lexeme()),
            })
        }
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Value> {
        if let Some(value) = self.values.get(name.lexeme()) {
            return Ok(value.clone());
        }

        if let Some(ref env) = self.enclosing {
            return env.borrow().get(name);
        }

        Err(RuntimeError {
            token: Some(name.clone()),
            message: format!("Undefined variable '{}'", name.lexeme()),
        })
    }
}
