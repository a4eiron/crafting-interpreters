use super::ResolveError;
use std::collections::HashMap;

use crate::lexer::{Literal, Token};
use crate::parser::*;
use crate::runtime::Interpreter;

pub type ResolveResult<T> = std::result::Result<T, ResolveError>;

#[derive(Debug)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(Debug)]
enum ClassType {
    None,
    Class,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve(&mut self, stmts: &[Stmt]) -> ResolveResult<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, statement: &Stmt) -> ResolveResult<()> {
        match statement {
            Stmt::Block(stmts) => self.resolve_block_stmt(stmts),
            Stmt::Class(stmt) => self.resolve_class_stmt(&stmt),
            Stmt::Expression(expr) => self.resolve_expr(expr),
            Stmt::Func(stmt) => self.resolve_func_stmt(stmt),
            Stmt::If(stmt) => self.resolve_if_stmt(stmt),
            Stmt::Print(expr) => self.resolve_expr(expr),
            Stmt::Return(stmt) => self.resolve_return_smt(stmt),
            Stmt::Var(stmt) => self.resolve_var_stmt(stmt),
            Stmt::While(stmt) => self.resolve_while_stmt(stmt),
            _ => Ok(()),
        }
    }

    fn resolve_block_stmt(&mut self, stmts: &[Stmt]) -> ResolveResult<()> {
        self.begin_scope();
        self.resolve(stmts)?;
        self.end_scope();
        Ok(())
    }

    fn resolve_var_stmt(&mut self, stmt: &VarStmt) -> ResolveResult<()> {
        self.declare(stmt.name.clone())?;
        if !matches!(stmt.initializer.kind, ExprKind::Literal(Literal::Nil)) {
            self.resolve_expr(&stmt.initializer)?;
        }
        self.define(stmt.name.clone());
        Ok(())
    }

    fn resolve_func_stmt(&mut self, stmt: &FuncStmt) -> ResolveResult<()> {
        self.declare(stmt.name.clone())?;
        self.define(stmt.name.clone());
        self.resolve_function(stmt, FunctionType::Function)?;
        Ok(())
    }

    fn resolve_if_stmt(&mut self, stmt: &IfStmt) -> ResolveResult<()> {
        self.resolve_expr(&stmt.condition)?;
        self.resolve_stmt(&stmt.then_branch)?;
        if let Some(else_branch) = &stmt.else_branch {
            self.resolve_stmt(&else_branch)?;
        }
        Ok(())
    }

    fn resolve_return_smt(&mut self, stmt: &ReturnStmt) -> ResolveResult<()> {
        match self.current_function {
            FunctionType::None => {
                return Err(ResolveError {
                    token: Some(stmt.keyword.clone()),
                    message: "cannot return from top-level code".to_string(),
                });
            }

            FunctionType::Initializer => {
                return Err(ResolveError {
                    token: Some(stmt.keyword.clone()),
                    message: "cannot return from initializer".to_string(),
                });
            }
            _ => {}
        }
        self.resolve_expr(&stmt.value)
    }

    fn resolve_while_stmt(&mut self, stmt: &WhileStmt) -> ResolveResult<()> {
        self.resolve_expr(&stmt.condition)?;
        self.resolve_stmt(&stmt.body)?;
        Ok(())
    }

    fn resolve_class_stmt(&mut self, stmt: &ClassStmt) -> ResolveResult<()> {
        let enclosing_class = std::mem::replace(&mut self.current_class, ClassType::Class);
        self.declare(stmt.name.clone())?;
        self.define(stmt.name.clone());

        if let Some(super_class) = &stmt.super_class {
            if let ExprKind::Var(expr) = &super_class.kind {
                if expr.token.lexeme() == stmt.name.lexeme() {
                    return Err(ResolveError {
                        token: Some(expr.token.clone()),
                        message: format!("a class cannot inherit from itself"),
                    });
                }
                self.resolve_var_expr(super_class, &expr.token)?;
            }
        }

        self.begin_scope();
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert("this".to_string(), true);
        }
        for method in &stmt.methods {
            let mut declaration = FunctionType::Method;
            if matches!(method.name.lexeme(), "init") {
                declaration = FunctionType::Initializer;
            }
            self.resolve_function(method, declaration)?;
        }

        self.current_class = enclosing_class;
        self.end_scope();

        Ok(())
    }

    fn resolve_expr(&mut self, expression: &Expr) -> ResolveResult<()> {
        match &expression.kind {
            ExprKind::Var(expr) => self.resolve_var_expr(expression, &expr.token),
            ExprKind::Assignment(expr) => self.resolve_assignment_expr(expression, expr),
            ExprKind::Unary(expr) => self.resolve_expr(&expr.right),
            ExprKind::Binary(expr) => self.resolve_binary_expr(expr),
            ExprKind::Call(expr) => self.resolve_call_expr(expr),
            ExprKind::Grouping(expr) => self.resolve_expr(&expr),
            ExprKind::Logical(expr) => self.resolve_logical_expr(expr),
            ExprKind::Get(expr) => self.resolve_expr(&expr.object),
            ExprKind::Set(expr) => self.resolve_set_expr(expr),
            ExprKind::This(keyword) => self.resolve_this(expression, keyword),
            _ => Ok(()),
        }
    }

    fn resolve_var_expr(&mut self, expression: &Expr, token: &Token) -> ResolveResult<()> {
        if let Some(scope) = self.scopes.last() {
            if scope.get(token.lexeme()).map_or(false, |defined| !defined) {
                return Err(ResolveError::new(
                    Some(token.clone()),
                    "cannot read local variable in its own initializer",
                ));
            }
        }
        self.resolve_local(expression, token);
        Ok(())
    }

    fn resolve_assignment_expr(
        &mut self,
        expression: &Expr,
        expr: &AssignmentExpr,
    ) -> ResolveResult<()> {
        self.resolve_expr(&expr.value)?;
        self.resolve_local(expression, &expr.name);
        Ok(())
    }

    fn resolve_binary_expr(&mut self, expr: &BinaryExpr) -> ResolveResult<()> {
        self.resolve_expr(&expr.left)?;
        self.resolve_expr(&expr.right)?;
        Ok(())
    }

    fn resolve_call_expr(&mut self, expr: &Call) -> ResolveResult<()> {
        self.resolve_expr(&expr.callee)?;
        for arg in &expr.args {
            self.resolve_expr(arg)?;
        }
        Ok(())
    }

    fn resolve_logical_expr(&mut self, expr: &LogicalExpr) -> ResolveResult<()> {
        self.resolve_expr(&expr.left)?;
        self.resolve_expr(&expr.right)?;
        Ok(())
    }

    fn resolve_this(&mut self, expression: &Expr, keyword: &Token) -> ResolveResult<()> {
        if matches!(self.current_class, ClassType::None) {
            return Err(ResolveError {
                token: Some(keyword.clone()),
                message: format!("cannot use 'this' outside of a class"),
            });
        }
        self.resolve_local(expression, keyword);
        Ok(())
    }
    fn resolve_set_expr(&mut self, expr: &SetExpr) -> ResolveResult<()> {
        self.resolve_expr(&expr.value)?;
        self.resolve_expr(&expr.object)?;
        Ok(())
    }

    fn resolve_local(&mut self, expression: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name.lexeme()) {
                self.interpreter.resolve(expression, i);
                return;
            }
        }
    }

    fn resolve_function(&mut self, func: &FuncStmt, func_type: FunctionType) -> ResolveResult<()> {
        let enclosing_function = std::mem::replace(&mut self.current_function, func_type);
        self.begin_scope();
        for param in &func.params {
            self.declare(param.clone())?;
            self.define(param.clone());
        }

        self.resolve(&func.body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: Token) -> ResolveResult<()> {
        if self.scopes.is_empty() {
            return Ok(());
        }
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(name.lexeme()) {
            return Err(ResolveError::new(
                Some(name),
                "already a varialbe with this name is in the scope",
            ));
        }

        scope.insert(name.lexeme().into(), false);
        Ok(())
    }

    fn define(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.last_mut().unwrap();
        scope
            .entry(name.lexeme().to_string())
            .and_modify(|f| *f = true);
    }
}
