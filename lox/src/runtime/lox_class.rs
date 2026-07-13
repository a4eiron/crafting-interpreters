use super::LoxFunction;

use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct LoxClass {
    name: String,
    super_class: Option<Rc<LoxClass>>,
    methods: HashMap<String, LoxFunction>,
    class_methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn new(
        name: &str,
        methods: HashMap<String, LoxFunction>,
        super_class: Option<Rc<LoxClass>>,
        class_methods: HashMap<String, LoxFunction>,
    ) -> Self {
        Self {
            name: name.into(),
            super_class,
            methods,
            class_methods,
        }
    }

    pub fn find_class_method(&self, name: &str) -> Option<LoxFunction> {
        self.class_methods
            .get(name)
            .cloned()
            .or_else(|| self.super_class.as_ref()?.find_class_method(name))
    }

    pub fn find_method(&self, name: &str) -> Option<LoxFunction> {
        self.methods
            .get(name)
            .cloned()
            .or_else(|| self.super_class.as_ref()?.find_method(name))
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}
