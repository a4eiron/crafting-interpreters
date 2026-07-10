use crate::lexer::{Literal, Token};

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Expr,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Expr,
    pub right: Expr,
    pub operator: Token,
}

#[derive(Debug, Clone)]
pub struct AssignmentExpr {
    pub name: Token,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct ConditionalExpr {
    pub condition: Expr,
    pub then_branch: Expr,
    pub else_branch: Expr,
}

#[derive(Debug, Clone)]
pub struct LogicalExpr {
    pub operator: Token,
    pub left: Expr,
    pub right: Expr,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub callee: Expr,
    pub paren: Token,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct GetExpr {
    pub object: Expr,
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct SetExpr {
    pub object: Expr,
    pub name: Token,
    pub value: Expr,
}

pub type ExprId = usize;

#[derive(Debug, Clone)]
pub struct Expr {
    pub id: ExprId,
    pub kind: ExprKind,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Var(Token),
    Literal(Literal),
    Grouping(Box<Expr>),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    Assignment(Box<AssignmentExpr>),
    Conditional(Box<ConditionalExpr>),
    Logical(Box<LogicalExpr>),
    Call(Box<Call>),
    Get(Box<GetExpr>),
    Set(Box<SetExpr>),
    This(Token),
}
