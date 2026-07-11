use crate::lexer::ScanError;
use crate::parser::ParseError;
use crate::resolver::ResolveError;
use crate::runtime::RuntimeError;

use std::fmt;

#[derive(Debug)]
pub enum LoxError {
    Io(std::io::Error),
    Scan(ScanError),
    Parse(ParseError),
    Resolve(ResolveError),
    Runtime(RuntimeError),
}

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::Scan(e) => write!(f, "{}", e),
            Self::Parse(e) => write!(f, "{}", e),
            Self::Resolve(e) => write!(f, "{}", e),
            Self::Runtime(e) => write!(f, "{}", e),
        }
    }
}

impl From<std::io::Error> for LoxError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<ScanError> for LoxError {
    fn from(e: ScanError) -> Self {
        Self::Scan(e)
    }
}

impl From<ParseError> for LoxError {
    fn from(e: ParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<ResolveError> for LoxError {
    fn from(e: ResolveError) -> Self {
        Self::Resolve(e)
    }
}

impl From<RuntimeError> for LoxError {
    fn from(e: RuntimeError) -> Self {
        Self::Runtime(e)
    }
}
