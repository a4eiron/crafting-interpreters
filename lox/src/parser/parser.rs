use super::*;

use crate::lexer::{Literal, Token, TokenType};

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
    next_expr_id: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: tokens,
            current: 0,
            next_expr_id: 0,
        }
    }

    fn expr(&mut self, kind: ExprKind) -> Expr {
        let expr = Expr {
            id: self.next_expr_id,
            kind,
        };
        self.next_expr_id += 1;
        expr
    }

    pub fn parse(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut stmts: Vec<Stmt> = Vec::new();

        while !self.at_end() {
            match self.declaration() {
                Ok(stmt) => stmts.push(stmt),
                Err(err) => {
                    eprintln!("{}", err);
                    self.synchronize();
                }
            }
        }
        Ok(stmts)
    }

    fn declaration(&mut self) -> ParseResult<Stmt> {
        if self.match_token(&[TokenType::Class]) {
            self.class_declaration()
        } else if self.match_token(&[TokenType::Func]) {
            self.func_declaration()
        } else if self.match_token(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.match_token(&[TokenType::If]) {
            self.if_stmt()
        } else if self.match_token(&[TokenType::Print]) {
            self.print_stmt()
        } else if self.match_token(&[TokenType::Break]) {
            self.break_stmt()
        } else if self.match_token(&[TokenType::Continue]) {
            self.continue_stmt()
        } else if self.match_token(&[TokenType::Return]) {
            self.return_stmt()
        } else if self.match_token(&[TokenType::For]) {
            self.for_stmt()
        } else if self.match_token(&[TokenType::While]) {
            self.while_stmt()
        } else if self.match_token(&[TokenType::LBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expr_stmt()
        }
    }

    fn break_stmt(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Break)
    }

    fn continue_stmt(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Continue)
    }

    fn return_stmt(&mut self) -> ParseResult<Stmt> {
        let token = self.previous().clone();
        let mut expr = self.expr(ExprKind::Literal(Literal::Nil));
        if !self.check(TokenType::Semicolon) {
            expr = self.expression()?;
        }

        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Return(ReturnStmt {
            keyword: token,
            value: expr,
        }))
    }

    fn for_stmt(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LParen)?;

        let initializer = if self.match_token(&[TokenType::Semicolon]) {
            None
        } else if self.match_token(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expr_stmt()?)
        };

        let mut condition: Option<Expr> = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(self.expression()?);
        }
        self.consume(TokenType::Semicolon)?;

        let mut increment: Option<Expr> = None;
        if !self.check(TokenType::RParen) {
            increment = Some(self.expression()?);
        }
        self.consume(TokenType::RParen)?;

        let mut stmt = self.statement()?;

        // if let Some(inc) = increment {
        //     body = Stmt::Block(vec![body, Stmt::Expression(inc)]);
        // }

        let condition = condition.unwrap_or(self.expr(ExprKind::Literal(Literal::True)));

        stmt = Stmt::While(WhileStmt {
            condition: condition,
            body: Box::new(stmt),
            increment: increment,
        });

        if let Some(init) = initializer {
            stmt = Stmt::Block(vec![init, stmt]);
        }

        Ok(stmt)
    }

    fn while_stmt(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LParen)?;
        let condition = self.expression()?;
        self.consume(TokenType::RParen)?;
        let body = self.statement()?;

        Ok(Stmt::While(WhileStmt {
            condition,
            body: Box::new(body),
            increment: None,
        }))
    }

    fn if_stmt(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LParen)?;
        let condition = self.expression()?;
        self.consume(TokenType::RParen)?;

        let then_branch = self.statement()?;

        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If(IfStmt {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch,
        }))
    }

    fn print_stmt(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Print(expr))
    }

    fn expr_stmt(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Expression(expr))
    }

    fn class_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::Identifier)?.clone();

        let super_class = if self.match_token(&[TokenType::Lesser]) {
            let token = self.consume(TokenType::Identifier)?.clone();
            Some(self.expr(ExprKind::Var(VarExpr { token })))
        } else {
            None
        };

        self.consume(TokenType::LBrace)?;

        let mut methods = Vec::new();
        let mut class_methods = Vec::new();

        while !self.check(TokenType::RBrace) && !self.at_end() {
            if self.match_token(&[TokenType::Class]) {
                if let Stmt::Func(class_func) = self.func_declaration()? {
                    class_methods.push(class_func);
                }
            } else {
                if let Stmt::Func(func_stmt) = self.func_declaration()? {
                    methods.push(func_stmt);
                }
            }
        }

        self.consume(TokenType::RBrace)?;

        Ok(Stmt::Class(ClassStmt {
            name,
            super_class,
            methods,
            class_methods,
        }))
    }

    fn func_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::Identifier)?.clone();
        self.consume(TokenType::LParen)?;

        let mut args = Vec::new();

        if !self.check(TokenType::RParen) {
            let mut t = true;
            while t {
                args.push(self.consume(TokenType::Identifier)?.clone());
                if !self.match_token(&[TokenType::Comma]) {
                    t = false;
                }
            }
        }

        self.consume(TokenType::RParen)?;
        self.consume(TokenType::LBrace)?;
        let body = self.block()?;

        Ok(Stmt::Func(FuncStmt {
            name,
            params: args,
            body,
        }))
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let identifier = self.consume(TokenType::Identifier)?.clone();
        let mut initializer = self.expr(ExprKind::Literal(Literal::Nil));

        if self.match_token(&[TokenType::Equal]) {
            initializer = self.expression()?;
        }
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Var(VarStmt {
            name: identifier,
            initializer: initializer,
        }))
    }

    fn block(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut stmts = Vec::new();

        while !self.check(TokenType::RBrace) && !self.at_end() {
            let stmt = self.declaration()?;
            stmts.push(stmt);
        }

        self.consume(TokenType::RBrace)?;
        Ok(stmts)
    }

    fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    fn or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.and()?;
        while self.match_token(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            expr = self.expr(ExprKind::Logical(Box::new(LogicalExpr {
                left: expr,
                right: right,
                operator,
            })));
        }
        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.conditional()?;

        while self.match_token(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.conditional()?;
            expr = self.expr(ExprKind::Logical(Box::new(LogicalExpr {
                left: expr,
                right: right,
                operator,
            })));
        }
        Ok(expr)
    }

    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.or()?;
        // let expr = self.conditional()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            match expr.kind {
                ExprKind::Get(get_expr) => {
                    return Ok(self.expr(ExprKind::Set(Box::new(SetExpr {
                        object: get_expr.object,
                        name: get_expr.name,
                        value: value,
                    }))));
                }
                ExprKind::Var(var_expr) => {
                    return Ok(self.expr(ExprKind::Assignment(Box::new(AssignmentExpr {
                        name: var_expr.token,
                        value: value,
                    }))));
                }
                _ => {
                    return Err(ParseError::InvalidAssignmentTarget {
                        line: equals.line(),
                    });
                }
            }
        }

        Ok(expr)
    }

    fn conditional(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::Question]) {
            let then_branch = self.expression()?;
            self.consume(TokenType::Colon)?;
            let else_branch = self.conditional()?;

            expr = self.expr(ExprKind::Conditional(Box::new(ConditionalExpr {
                condition: expr,
                then_branch: then_branch,
                else_branch: else_branch,
            })));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = self.expr(ExprKind::Binary(Box::new(BinaryExpr {
                left: expr,
                right: right,
                operator: operator,
            })));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Lesser,
            TokenType::LesserEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;

            expr = self.expr(ExprKind::Binary(Box::new(BinaryExpr {
                left: expr,
                right: right,
                operator: operator,
            })));
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;

            expr = self.expr(ExprKind::Binary(Box::new(BinaryExpr {
                left: expr,
                right: right,
                operator: operator,
            })));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = self.expr(ExprKind::Binary(Box::new(BinaryExpr {
                left: expr,
                right: right,
                operator: operator,
            })));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(self.expr(ExprKind::Unary(Box::new(UnaryExpr {
                operator,
                right: right,
            }))));
        }
        self.call()
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenType::Dot]) {
                let name = self.consume(TokenType::Identifier)?.clone();
                expr = self.expr(ExprKind::Get(Box::new(GetExpr { object: expr, name })))
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, expr: Expr) -> ParseResult<Expr> {
        let mut args = vec![];

        if !self.check(TokenType::RParen) {
            let mut t = true;
            while t {
                // i don't wanna have a limit, skipping..
                // if args.len() >= 255 {
                //     return Err(ParseError {
                //         line: self.peek().line(),
                //         message: format!("cannot have more than 255 arguments"),
                //     });
                // }
                args.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    t = false;
                }
            }
        }

        let paren = self.consume(TokenType::RParen)?.clone();

        Ok(self.expr(ExprKind::Call(Box::new(Call {
            callee: expr,
            paren: paren,
            args,
        }))))
    }

    fn primary(&mut self) -> ParseResult<Expr> {
        if self.match_token(&[TokenType::False]) {
            return Ok(self.expr(ExprKind::Literal(Literal::False)));
            // for space
        } else if self.match_token(&[TokenType::True]) {
            return Ok(self.expr(ExprKind::Literal(Literal::True)));
            //
        } else if self.match_token(&[TokenType::Nil]) {
            return Ok(self.expr(ExprKind::Literal(Literal::Nil)));
            //
        } else if self.match_token(&[TokenType::Number, TokenType::String]) {
            if let Some(literal) = self.previous().literal().cloned() {
                return Ok(self.expr(ExprKind::Literal(literal)));
            }
            //
        } else if self.match_token(&[TokenType::This]) {
            return Ok(self.expr(ExprKind::This(self.previous().clone())));
            //
        } else if self.match_token(&[TokenType::Super]) {
            let keyword = self.previous().clone();
            self.consume(TokenType::Dot)?;
            let method = self.consume(TokenType::Identifier)?.clone();
            return Ok(self.expr(ExprKind::Super(SuperExpr { keyword, method })));
            //
        } else if self.match_token(&[TokenType::Identifier]) {
            let token = self.previous().clone();
            return Ok(self.expr(ExprKind::Var(VarExpr { token })));
            //
        } else if self.match_token(&[TokenType::LParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RParen)?;
            return Ok(self.expr(ExprKind::Grouping(Box::new(expr))));
        }

        return Err(ParseError::UnexpectedToken {
            line: self.peek().line(),
            found: self.peek().token_type(),
        });
    }

    fn consume(&mut self, token_type: TokenType) -> ParseResult<&Token> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        return Err(ParseError::ExpectedToken {
            line: self.peek().line(),
            found: self.peek().token_type(),
            expected: token_type,
        });
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.at_end() {
            if self.previous().token_type() == TokenType::Semicolon {
                return;
            }

            if matches!(
                self.peek().token_type(),
                TokenType::Class
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return
            ) {
                return;
            }
            self.advance();
        }
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        !self.at_end() && self.peek().token_type() == token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.current += 1;
        }
        return self.previous();
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn at_end(&self) -> bool {
        self.peek().token_type() == TokenType::Eof
    }
}
