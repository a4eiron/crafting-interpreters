use super::expression::Expr;
use crate::lexer::Token;

#[derive(Debug, Clone)]
pub struct VarStmt {
    pub name: Token,
    pub initializer: Expr,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct FuncStmt {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub keyword: Token,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Break,
    Print(Expr),
    If(IfStmt),
    Var(VarStmt),
    Func(FuncStmt),
    Expression(Expr),
    Block(Vec<Stmt>),
    While(WhileStmt),
    Return(ReturnStmt),
}
