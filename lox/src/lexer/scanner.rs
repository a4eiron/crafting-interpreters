use super::*;

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> ScannerResult<&[Token]> {
        while !self.at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens
            .push(Token::new(TokenType::Eof, self.line, "".to_string(), None));

        Ok(&self.tokens)
    }

    fn scan_token(&mut self) -> ScannerResult<()> {
        let c = match self.advance() {
            Some(character) => character,
            None => return Ok(()),
        };

        match c {
            '(' => self.add_token(TokenType::LParen),
            ')' => self.add_token(TokenType::RParen),
            '{' => self.add_token(TokenType::LBrace),
            '}' => self.add_token(TokenType::RBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '+' => self.add_token(TokenType::Plus),
            '-' => self.add_token(TokenType::Minus),
            '*' => self.add_token(TokenType::Star),
            ';' => self.add_token(TokenType::Semicolon),
            '?' => self.add_token(TokenType::Question),
            ':' => self.add_token(TokenType::Colon),
            '!' => self.add_token_if_match('=', TokenType::BangEqual, TokenType::Bang),
            '=' => self.add_token_if_match('=', TokenType::EqualEqual, TokenType::Equal),
            '<' => self.add_token_if_match('=', TokenType::LesserEqual, TokenType::Lesser),
            '>' => self.add_token_if_match('=', TokenType::GreaterEqual, TokenType::Greater),
            '/' => self.scan_slash()?,
            '"' => self.scan_string()?,
            '0'..='9' => self.scan_number()?,
            'a'..='z' | 'A'..='Z' | '_' => self.scan_identifier(),
            '\n' => self.line += 1,
            ' ' | '\t' | '\r' => {}
            _ => return Err(ScanError::UnexpectedChar { line: self.line, c }),
        }

        Ok(())
    }

    fn scan_identifier(&mut self) {
        while self.peek().is_some_and(|c| c.is_alphanumeric()) {
            self.advance();
        }

        let token = self.current_lexeme();

        let token_type = match token {
            "var" => TokenType::Var,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "return" => TokenType::Return,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "nil" => TokenType::Nil,
            "print" => TokenType::Print,
            "class" => TokenType::Class,
            "this" => TokenType::This,
            "super" => TokenType::Super,
            "func" => TokenType::Func,
            _ => TokenType::Identifier,
        };

        self.add_token(token_type);
    }

    fn scan_number(&mut self) -> ScannerResult<()> {
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }

        if self.peek() == Some('.') && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
        }

        let lexeme = self.current_lexeme();
        let number = lexeme
            .parse()
            .map_err(|_| ScanError::InvalidNumber { line: self.line })?;
        self.add_token_with_literal(TokenType::Number, Some(Literal::Number(number)));

        Ok(())
    }

    fn scan_string(&mut self) -> ScannerResult<()> {
        while self.peek() != Some('"') && !self.at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }

        if self.at_end() {
            return Err(ScanError::UnterminatedStr { line: self.line });
        }
        self.advance();
        self.add_token_with_literal(
            TokenType::String,
            Some(Literal::String(
                self.source[self.start + 1..self.current - 1].to_string(),
            )),
        );
        Ok(())
    }

    fn scan_slash(&mut self) -> ScannerResult<()> {
        if self.match_char('/') {
            while self.peek() != Some('\n') && !self.at_end() {
                self.advance();
            }
        } else if self.match_char('*') {
            while !self.at_end() {
                if self.peek() == Some('\n') {
                    self.line += 1;
                }

                if self.match_char('*') {
                    if self.match_char('/') {
                        return Ok(());
                    }
                    continue;
                }

                self.advance();
            }

            return Err(ScanError::UnterminatedComment { line: self.line });
        } else {
            self.add_token(TokenType::Slash);
        }

        Ok(())
    }

    fn current_lexeme(&self) -> &str {
        &self.source[self.start..self.current]
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None);
    }

    fn add_token_if_match(&mut self, expected: char, if_match: TokenType, otherwise: TokenType) {
        if self.match_char(expected) {
            self.add_token(if_match);
        } else {
            self.add_token(otherwise);
        }
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let lexeme = self.current_lexeme();
        self.tokens.push(Token::new(
            token_type,
            self.line,
            lexeme.to_string(),
            literal,
        ));
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.current += c.len_utf8();
        Some(c)
    }

    fn peek(&self) -> Option<char> {
        self.source[self.current..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.current..].chars();
        chars.next();
        chars.next()
    }

    fn at_end(&self) -> bool {
        self.peek().is_none()
    }
}
