use std::fmt;

pub type ScannerResult<T> = std::result::Result<T, ScanError>;

#[derive(Debug)]
pub enum ScanError {
    UnexpectedChar { line: usize, c: char },
    UnterminatedStr { line: usize },
    UnterminatedComment { line: usize },
    InvalidNumber { line: usize },
}

impl std::error::Error for ScanError {}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedChar { line, c } => {
                write!(f, "line: {} | Unknown character: {}", line, c)
            }
            Self::UnterminatedStr { line } => {
                write!(f, "line: {} | Non-terminating string", line)
            }
            Self::UnterminatedComment { line } => {
                write!(f, "line: {} | Unterminated Comment", line)
            }
            Self::InvalidNumber { line } => {
                write!(f, "line: {} | InvalidNumber", line)
            }
        }
    }
}
