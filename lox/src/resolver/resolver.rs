use super::ResolveError;
use std::collections::HashMap;

use crate::lexer::{Literal, Token};
use crate::parser::{Expr, ExprKind, FuncStmt, Stmt};
use crate::runtime::Interpreter;

pub type ResolveResult<T> = std::result::Result<T, ResolveError>;

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
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
            Stmt::Block(stmts) => {
                self.begin_scope();
                self.resolve(stmts)?;
                self.end_scope();
                Ok(())
            }
            Stmt::Var(stmt) => {
                self.declare(stmt.name.clone())?;
                if !matches!(stmt.initializer.kind, ExprKind::Literal(Literal::Nil)) {
                    self.resolve_expr(&stmt.initializer)?;
                }
                self.define(stmt.name.clone());
                Ok(())
            }
            Stmt::Func(stmt) => {
                self.declare(stmt.name.clone())?;
                self.define(stmt.name.clone());
                self.resolve_function(stmt)?;
                Ok(())
            }
            Stmt::If(stmt) => {
                self.resolve_expr(&stmt.condition)?;
                self.resolve_stmt(&stmt.then_branch)?;
                if let Some(else_branch) = &stmt.else_branch {
                    self.resolve_stmt(&else_branch)?;
                }
                Ok(())
            }
            Stmt::Expression(expr) => self.resolve_expr(expr),
            Stmt::Print(expr) => self.resolve_expr(expr),
            Stmt::Return(stmt) => self.resolve_expr(&stmt.value),
            Stmt::While(stmt) => {
                self.resolve_expr(&stmt.condition)?;
                self.resolve_stmt(&stmt.body)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn resolve_expr(&mut self, expression: &Expr) -> ResolveResult<()> {
        match &expression.kind {
            ExprKind::Var(token) => {
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

            ExprKind::Assignment(expr) => {
                self.resolve_expr(&expr.value)?;
                self.resolve_local(expression, &expr.name);
                Ok(())
            }
            ExprKind::Unary(expr) => self.resolve_expr(&expr.right),
            ExprKind::Binary(expr) => {
                self.resolve_expr(&expr.left)?;
                self.resolve_expr(&expr.right)?;
                Ok(())
            }
            ExprKind::Call(expr) => {
                self.resolve_expr(&expr.callee)?;
                for arg in &expr.args {
                    self.resolve_expr(arg)?;
                }
                Ok(())
            }
            ExprKind::Grouping(expr) => self.resolve_expr(&expr),
            ExprKind::Logical(expr) => {
                self.resolve_expr(&expr.left)?;
                self.resolve_expr(&expr.right)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn resolve_local(&mut self, expression: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name.lexeme()) {
                self.interpreter.resolve(expression, i);
                return;
            }
        }
    }

    fn resolve_function(&mut self, func: &FuncStmt) -> ResolveResult<()> {
        self.begin_scope();
        for param in &func.params {
            self.declare(param.clone())?;
            self.define(param.clone());
        }

        self.resolve(&func.body)?;
        self.end_scope();
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
