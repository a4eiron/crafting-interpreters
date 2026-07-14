use std::{fmt, rc::Rc};

use crate::runtime::*;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Callable(Rc<dyn Callable>),
    Class(Rc<LoxClass>),
    Instance(LoxInstance),
}

impl Value {
    pub fn add(self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + b)),
            (_, _) => Err(RuntimeError::new("operands must be numbers or strings")),
        }
    }

    pub fn sub(self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            (_, _) => Err(RuntimeError::new("operands must be numbers")),
        }
    }

    pub fn mul(self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            (_, _) => Err(RuntimeError::new("operands must be numbers")),
        }
    }

    pub fn divide(self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
            (_, _) => Err(RuntimeError::new("operands must be numbers")),
        }
    }

    pub fn equal(&self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a == b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a == b)),
            (a, Value::Bool(b)) => Ok(Value::Bool(is_truthy(a) == *b)),
            (Value::Bool(a), b) => Ok(Value::Bool(*a == is_truthy(b))),
            (Value::Nil, Value::Nil) => Ok(Value::Bool(true)),
            (_, _) => Err(RuntimeError::new("incompatable")),
        }
    }

    pub fn not_equal(&self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a != b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a != b)),
            (a, Value::Bool(b)) => Ok(Value::Bool(is_truthy(a) != *b)),
            (Value::Bool(a), b) => Ok(Value::Bool(*a != is_truthy(b))),
            (_, _) => Err(RuntimeError::new("incompatable")),
        }
    }

    pub fn greater(&self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
            (_, _) => Err(RuntimeError::new("incompatable")),
        }
    }

    pub fn greater_equal(&self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
            (_, _) => Err(RuntimeError::new("incompatable")),
        }
    }

    pub fn lesser(&self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
            (_, _) => Err(RuntimeError::new("incompatable")),
        }
    }

    pub fn lesser_equal(&self, rhs: &Value) -> RuntimeResult<Value> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
            (_, _) => Err(RuntimeError::new("incompatable")),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Callable(func) => write!(f, "{}", func),
            Self::Class(class) => write!(f, "{}", class),
            Self::Instance(instance) => write!(f, "{}", instance),
        }
    }
}
