use crate::lexer::TokenType;

use std::fmt;

pub type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        line: usize,
        found: TokenType,
    },
    ExpectedToken {
        line: usize,
        found: TokenType,
        expected: TokenType,
    },
    InvalidAssignmentTarget {
        line: usize,
    },
}

impl std::error::Error for ParseError {}

impl ParseError {
    pub fn line(&self) -> usize {
        match self {
            Self::UnexpectedToken { line, .. }
            | Self::ExpectedToken { line, .. }
            | Self::InvalidAssignmentTarget { line } => *line,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken { line, found } => {
                write!(f, "line: {} | unexpected token {}", line, found)
            }
            Self::ExpectedToken {
                line,
                found,
                expected,
            } => write!(f, "line: {} | expected {} found {}", line, expected, found),
            Self::InvalidAssignmentTarget { line } => {
                write!(f, "line: {} | invalid assignment target", line)
            }
        }
    }
}
