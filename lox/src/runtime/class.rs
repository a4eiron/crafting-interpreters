use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::lexer::Token;
use crate::runtime::{LoxFunction, RuntimeError, RuntimeResult, Value};

#[derive(Debug)]
pub struct LoxClass {
    name: String,
    super_class: Option<Rc<LoxClass>>,
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClass {
    pub fn new(name: &str, methods: HashMap<String, Rc<LoxFunction>>) -> Self {
        Self {
            name: name.to_string(),
            super_class: None,
            methods: methods,
        }
    }

    pub fn new_with_super_class(
        name: &str,
        methods: HashMap<String, Rc<LoxFunction>>,
        super_class: Option<Rc<LoxClass>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            super_class: super_class,
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<LoxFunction>> {
        match self.methods.get(name) {
            Some(method) => Some(method.clone()),
            None => {
                if let Some(super_class) = &self.super_class {
                    return super_class.find_method(name);
                }
                None
            }
        }
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    class: Rc<LoxClass>,
    fields: HashMap<String, Value>,
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Value> {
        if self.fields.contains_key(name.lexeme()) {
            let value = self.fields.get(name.lexeme()).unwrap();
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
    pub fn set(&mut self, name: Token, value: Value) -> RuntimeResult<()> {
        self.fields.insert(name.lexeme().into(), value);
        Ok(())
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "instance of {}", self.class)
    }
}
