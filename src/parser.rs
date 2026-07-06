// TODO: Rc instead of cloning everywhere..(when the time comes)
use std::fmt::{self};

use crate::token::*;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    // token_type: TokenType,
    line: usize,
    message: String,
}

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line: {} | {}", self.line, self.message)
    }
}

#[derive(Debug)]
pub enum Expr {
    Literal(Literal),
    Grouping(Box<Expr>),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        right: Box<Expr>,
        operator: Token,
    },
    Conditional {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Var(Token),
}

#[derive(Debug)]
pub enum Stmt {
    Print(Expr),
    Expression(Expr),
    Var { name: Token, initializer: Expr },
}

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

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
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

    fn declaration(&mut self) -> Result<Stmt> {
        if self.match_token(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_token(&[TokenType::Print]) {
            self.print_stmt()
        } else {
            self.expr_stmt()
        }
    }

    fn print_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Print(expr))
    }

    fn expr_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Expression(expr))
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let identifier = self.consume(TokenType::Identifier)?;
        let mut initializer = Expr::Literal(Literal::Nil);

        if self.match_token(&[TokenType::Equal]) {
            initializer = self.expression()?;
        }
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Var {
            name: identifier,
            initializer: initializer,
        })
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.conditional()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            match expr {
                Expr::Var(token) => {
                    return Ok(Expr::Assignment {
                        name: token,
                        value: Box::new(value),
                    });
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

    fn conditional(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::Question]) {
            let then_branch = self.expression()?;
            self.consume(TokenType::Colon)?;
            let else_branch = self.conditional()?;

            expr = Expr::Conditional {
                condition: Box::new(expr),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            }
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator: operator,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Lesser,
            TokenType::LesserEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
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

    fn consume(&mut self, token_type: TokenType) -> Result<Token> {
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
