use crate::lexer::Token;
use std::fmt;

pub type ResolveResult<T> = std::result::Result<T, ResolveError>;

#[derive(Debug)]
pub enum ResolveError {
    ReturnFromTopLevel(Token),
    ReturnFromInitializer(Token),
    ClassInheritsFromItself(Token),
    ReadLocalInOwnInitializer(Token),
    SuperOutsideClass(Token),
    SuperWithoutSuperClass(Token),
    ThisOutsideClass(Token),
    VariableAlreadyInScope(Token),
    BreakOutsideLoop(Token),
    ContinueOutsideLoop(Token),
}

impl std::error::Error for ResolveError {}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::ReturnFromTopLevel(..) => "cannot return from top-level code".into(),
            Self::ReturnFromInitializer(..) => "cannot return from an initializer".into(),
            Self::ClassInheritsFromItself(..) => "a class cannot inherit from itself".into(),
            Self::ReadLocalInOwnInitializer(..) => {
                "cannot read local variable into its own initializer".into()
            }
            Self::SuperOutsideClass(..) => "cannot use super outside a class".into(),
            Self::SuperWithoutSuperClass(..) => "cannot use super without a super classa".into(),
            Self::ThisOutsideClass(..) => "cannot use 'this' outside a class".into(),
            Self::VariableAlreadyInScope(token) => {
                format!("variable {} already in scope", token.lexeme())
            }
            Self::ContinueOutsideLoop(..) => "cannot use continue outside a loop".into(),
            Self::BreakOutsideLoop(..) => "cannot use break outside a loop".into(),
        };

        write!(f, "line: {} | {}", self.token().line(), msg)
    }
}

impl ResolveError {
    pub fn token(&self) -> &Token {
        match self {
            Self::ReturnFromTopLevel(token)
            | Self::ReturnFromInitializer(token)
            | Self::ClassInheritsFromItself(token)
            | Self::ReadLocalInOwnInitializer(token)
            | Self::SuperOutsideClass(token)
            | Self::SuperWithoutSuperClass(token)
            | Self::ThisOutsideClass(token)
            | Self::ContinueOutsideLoop(token)
            | Self::BreakOutsideLoop(token)
            | Self::VariableAlreadyInScope(token) => token,
        }
    }
}
