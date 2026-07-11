use super::LoxFunction;

use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

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
