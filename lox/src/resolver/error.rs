use crate::lexer::Token;
use std::fmt;

#[derive(Debug)]
pub struct ResolveError {
    pub token: Option<Token>,
    pub message: String,
}

impl std::error::Error for ResolveError {}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.token {
            Some(token) => write!(f, "line: {} | {}", token.line(), self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl ResolveError {
    pub fn new(token: Option<Token>, msg: impl Into<String>) -> Self {
        Self {
            token: token,
            message: msg.into(),
        }
    }
}
