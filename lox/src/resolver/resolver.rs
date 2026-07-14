use super::*;
use crate::lexer::{Literal, Token};
use crate::parser::*;
use crate::runtime::Interpreter;

use std::collections::HashMap;

#[derive(Debug)]
enum FunctionType {
    None,
    Function,
    Method,
    ClassMethod,
    Initializer,
}

#[derive(Debug)]
enum ClassType {
    None,
    Class,
    SubClass,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
    loop_depth: usize,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
            loop_depth: 0,
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
            Stmt::Class(stmt) => self.resolve_class_stmt(stmt),
            Stmt::Expression(expr) => self.resolve_expr(expr),
            Stmt::Function(stmt) => self.resolve_func_stmt(stmt),
            Stmt::If(stmt) => self.resolve_if_stmt(stmt),
            Stmt::Print(expr) => self.resolve_expr(expr),
            Stmt::Return(stmt) => self.resolve_return_smt(stmt),
            Stmt::Var(stmt) => self.resolve_var_stmt(stmt),
            Stmt::While(stmt) => self.resolve_while_stmt(stmt),
            Stmt::Break(token) if self.loop_depth == 0 => {
                Err(ResolveError::BreakOutsideLoop(token.clone()))
            }
            Stmt::Continue(token) if self.loop_depth == 0 => {
                Err(ResolveError::ContinueOutsideLoop(token.clone()))
            }
            Stmt::Break(_) | Stmt::Continue(_) => Ok(()),
        }
    }

    fn resolve_expr(&mut self, expression: &Expr) -> ResolveResult<()> {
        match &expression.kind {
            ExprKind::Assignment(expr) => self.resolve_assignment_expr(expression, expr),
            ExprKind::Binary(expr) => self.resolve_binary_expr(expr),
            ExprKind::Call(expr) => self.resolve_call_expr(expr),
            ExprKind::Function(func_expr) => self.resolve_func_expr(func_expr),
            ExprKind::Grouping(expr) => self.resolve_expr(expr),
            ExprKind::Get(expr) => self.resolve_expr(&expr.object),
            ExprKind::Logical(expr) => self.resolve_logical_expr(expr),
            ExprKind::Set(expr) => self.resolve_set_expr(expr),
            ExprKind::Super(super_expr) => self.resolve_super_expr(expression, super_expr),
            ExprKind::This(keyword) => self.resolve_this(expression, keyword),
            ExprKind::Unary(expr) => self.resolve_expr(&expr.right),
            ExprKind::Var(expr) => self.resolve_var_expr(expression, &expr.token),
            _ => Ok(()),
        }
    }

    fn resolve_block_stmt(&mut self, stmts: &[Stmt]) -> ResolveResult<()> {
        self.begin_scope();
        self.resolve(stmts)?;
        self.end_scope();
        Ok(())
    }

    fn resolve_class_stmt(&mut self, stmt: &ClassStmt) -> ResolveResult<()> {
        let enclosing_class = std::mem::replace(&mut self.current_class, ClassType::Class);
        self.declare(stmt.name.clone())?;
        self.define(stmt.name.clone());

        if let Some(super_class) = &stmt.super_class {
            self.current_class = ClassType::SubClass;
            self.begin_scope();
            self.scopes
                .last_mut()
                .unwrap()
                .insert("super".to_string(), true);

            if let ExprKind::Var(expr) = &super_class.kind {
                if expr.token.lexeme() == stmt.name.lexeme() {
                    return Err(ResolveError::ClassInheritsFromItself(expr.token.clone()));
                }
                self.resolve_var_expr(super_class, &expr.token)?;
            }
        }

        for class_method in &stmt.class_methods {
            self.resolve_function(class_method, FunctionType::ClassMethod)?;
        }

        self.begin_scope();
        self.scopes
            .last_mut()
            .unwrap()
            .insert("this".to_string(), true);

        for method in &stmt.methods {
            let declaration = if method.name.lexeme() == "init" {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            };
            self.resolve_function(method, declaration)?;
        }
        self.end_scope();

        if stmt.super_class.is_some() {
            self.end_scope();
        }
        self.current_class = enclosing_class;
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
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn resolve_return_smt(&mut self, stmt: &ReturnStmt) -> ResolveResult<()> {
        match self.current_function {
            FunctionType::None => {
                return Err(ResolveError::ReturnFromTopLevel(stmt.keyword.clone()));
            }

            FunctionType::Initializer => {
                return Err(ResolveError::ReturnFromInitializer(stmt.keyword.clone()));
            }
            _ => {}
        }
        self.resolve_expr(&stmt.value)
    }

    fn resolve_while_stmt(&mut self, stmt: &WhileStmt) -> ResolveResult<()> {
        self.resolve_expr(&stmt.condition)?;
        self.loop_depth += 1;
        self.resolve_stmt(&stmt.body)?;
        self.loop_depth -= 1;
        if let Some(incr) = &stmt.increment {
            self.resolve_expr(incr)?;
        }
        Ok(())
    }

    fn resolve_func_expr(&mut self, func_expr: &FunctionExpr) -> ResolveResult<()> {
        self.resolve_function_body(&func_expr.params, &func_expr.body, FunctionType::Function)
    }

    fn resolve_super_expr(
        &mut self,
        expression: &Expr,
        super_expr: &SuperExpr,
    ) -> ResolveResult<()> {
        if matches!(self.current_class, ClassType::None) {
            return Err(ResolveError::SuperOutsideClass(super_expr.keyword.clone()));
        }

        if !matches!(self.current_class, ClassType::SubClass) {
            return Err(ResolveError::SuperWithoutSuperClass(
                super_expr.keyword.clone(),
            ));
        }

        self.resolve_local(expression, &super_expr.keyword);
        Ok(())
    }

    fn resolve_var_expr(&mut self, expression: &Expr, token: &Token) -> ResolveResult<()> {
        if let Some(scope) = self.scopes.last()
            && scope.get(token.lexeme()).is_some_and(|defined| !defined)
        {
            return Err(ResolveError::ReadLocalInOwnInitializer(token.clone()));
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
            return Err(ResolveError::ThisOutsideClass(keyword.clone()));
        }
        self.resolve_local(expression, keyword);
        Ok(())
    }

    fn resolve_set_expr(&mut self, expr: &SetExpr) -> ResolveResult<()> {
        self.resolve_expr(&expr.value)?;
        self.resolve_expr(&expr.object)?;
        Ok(())
    }

    fn resolve_function(&mut self, func: &FuncStmt, func_type: FunctionType) -> ResolveResult<()> {
        self.resolve_function_body(&func.params, &func.body, func_type)
    }

    fn resolve_local(&mut self, expression: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name.lexeme()) {
                self.interpreter.resolve(expression, i);
                return;
            }
        }
    }

    fn resolve_function_body(
        &mut self,
        params: &[Token],
        body: &[Stmt],
        func_type: FunctionType,
    ) -> ResolveResult<()> {
        let enclosing_function = std::mem::replace(&mut self.current_function, func_type);
        let enclosing_loop_depth = self.loop_depth;
        self.loop_depth = 0;

        self.begin_scope();
        for param in params {
            self.declare(param.clone())?;
            self.define(param.clone());
        }

        self.resolve(body)?;
        self.end_scope();
        self.loop_depth = enclosing_loop_depth;
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
            return Err(ResolveError::VariableAlreadyInScope(name));
        }

        scope.insert(name.lexeme().into(), false);
        Ok(())
    }

    fn define(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }
        self.scopes
            .last_mut()
            .unwrap()
            .entry(name.lexeme().into())
            .and_modify(|f| *f = true);
    }
}
