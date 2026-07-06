use std::collections::HashMap;

use crate::interpreter::{Result, RuntimeError, Value};
use crate::token::Token;

pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<&Value> {
        if let Some(value) = self.values.get(name.lexeme()) {
            return Ok(value);
        }
        Err(RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'", name.lexeme()),
        })
    }
}
