// TODO: Rc instead of cloning everywhere..(when the time comes)
use std::fmt::{self};

use crate::token::*;

/////////////////////////////////////////////////////////////////////////////////////////
pub type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    // token_type: TokenType,
    line: usize,
    message: String,
}

impl std::error::Error for ParseError {}

impl ParseError {
    pub fn new(line: usize, message: &str) -> Self {
        Self {
            line,
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line: {} | {}", self.line, self.message)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
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
pub enum Expr {
    Var(Token),
    Literal(Literal),
    Grouping(Box<Expr>),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    Assignment(Box<AssignmentExpr>),
    Conditional(Box<ConditionalExpr>),
    Logical(Box<LogicalExpr>),
    Call(Box<Call>),
}

/////////////////////////////////////////////////////////////////////////////////////////
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
    pub args: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub keyword: Token,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Expr),
    If(IfStmt),
    Var(VarStmt),
    Func(FuncStmt),
    Expression(Expr),
    Block(Vec<Stmt>),
    While(WhileStmt),
    Return(ReturnStmt),
}

/////////////////////////////////////////////////////////////////////////////////////////
pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: tokens,
            current: 0,
        }
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
        if self.match_token(&[TokenType::Func]) {
            self.function("function")
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

    fn return_stmt(&mut self) -> ParseResult<Stmt> {
        let token = self.previous();
        let mut expr = Expr::Literal(Literal::Nil);
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

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(inc)]);
        }

        let condition = condition.unwrap_or(Expr::Literal(Literal::True));

        body = Stmt::While(WhileStmt {
            condition: condition,
            body: Box::new(body),
        });

        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body]);
        }

        Ok(body)
    }

    fn while_stmt(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LParen)?;
        let condition = self.expression()?;
        self.consume(TokenType::RParen)?;
        let body = self.statement()?;

        Ok(Stmt::While(WhileStmt {
            condition,
            body: Box::new(body),
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

    fn function(&mut self, identifier: &str) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::Identifier)?;
        self.consume(TokenType::LParen)?;

        let mut args = Vec::new();

        if !self.check(TokenType::RParen) {
            let mut t = true;
            while t {
                args.push(self.consume(TokenType::Identifier)?);
                if !self.match_token(&[TokenType::Comma]) {
                    t = false;
                }
            }
        }

        self.consume(TokenType::RParen)?;
        self.consume(TokenType::LBrace)?;
        let body = self.block()?;

        Ok(Stmt::Func(FuncStmt { name, args, body }))
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let identifier = self.consume(TokenType::Identifier)?;
        let mut initializer = Expr::Literal(Literal::Nil);

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
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical(Box::new(LogicalExpr {
                left: expr,
                right: right,
                operator,
            }));
        }
        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.conditional()?;

        while self.match_token(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.conditional()?;
            expr = Expr::Logical(Box::new(LogicalExpr {
                left: expr,
                right: right,
                operator,
            }));
        }
        Ok(expr)
    }

    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.or()?;
        // let expr = self.conditional()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            match expr {
                Expr::Var(token) => {
                    return Ok(Expr::Assignment(Box::new(AssignmentExpr {
                        name: token,
                        value: value,
                    })));
                }
                _ => {
                    return Err(ParseError {
                        // token_type: equals.token_type(),
                        line: equals.line(),
                        message: format!("invalid assigment target"),
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

            expr = Expr::Conditional(Box::new(ConditionalExpr {
                condition: expr,
                then_branch: then_branch,
                else_branch: else_branch,
            }));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(BinaryExpr {
                left: expr,
                right: right,
                operator: operator,
            }));
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
            let operator = self.previous();
            let right = self.term()?;

            expr = Expr::Binary(Box::new(BinaryExpr {
                left: expr,
                right: right,
                operator: operator,
            }));
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;

            expr = Expr::Binary(Box::new(BinaryExpr {
                left: expr,
                right: right,
                operator: operator,
            }));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(BinaryExpr {
                left: expr,
                right: right,
                operator: operator,
            }));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(Box::new(UnaryExpr {
                operator,
                right: right,
            })));
        }
        self.call()
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, expr: Expr) -> ParseResult<Expr> {
        let mut args = vec![];

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

        let paren = self.consume(TokenType::RParen)?;

        Ok(Expr::Call(Box::new(Call {
            callee: expr,
            paren: paren,
            args,
        })))
    }

    fn primary(&mut self) -> ParseResult<Expr> {
        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal(Literal::False));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal(Literal::True));
        }

        if self.match_token(&[TokenType::Nil]) {
            return Ok(Expr::Literal(Literal::Nil));
        }

        if self.match_token(&[TokenType::Number, TokenType::String]) {
            if let Some(literal) = self.previous().literal().cloned() {
                return Ok(Expr::Literal(literal));
            }
        }

        if self.match_token(&[TokenType::Identifier]) {
            return Ok(Expr::Var(self.previous()));
        }

        if self.match_token(&[TokenType::LParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RParen)?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        return Err(ParseError {
            // token_type: self.peek().token_type(),
            line: self.peek().line(),
            message: format!("unexpected token {}", self.peek().token_type()),
        });
    }

    fn consume(&mut self, token_type: TokenType) -> ParseResult<Token> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        return Err(ParseError {
            // token_type,
            line: self.peek().line(),
            message: format!("expected {} found {}", token_type, self.peek().token_type()),
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

    fn advance(&mut self) -> Token {
        if !self.at_end() {
            self.current += 1;
        }
        return self.previous();
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn at_end(&self) -> bool {
        self.peek().token_type() == TokenType::Eof
    }
}
