use crate::lexer::Token;
use crate::runtime::ControlFlow;

use std::fmt;

impl From<RuntimeError> for ControlFlow {
    fn from(err: RuntimeError) -> Self {
        ControlFlow::Error(err)
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Option<Token>,
    pub message: String,
}

impl std::error::Error for RuntimeError {}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.token {
            Some(token) => write!(f, "line: {} | {}", token.line(), self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl RuntimeError {
    pub fn new(msg: &str) -> Self {
        Self {
            token: None,
            message: msg.to_string(),
        }
    }
}
