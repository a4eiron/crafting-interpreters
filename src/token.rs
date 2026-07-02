#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Plus,
    Minus,
    Star,
    Slash,
    Dot,
    Comma,
    Semicolon,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Lesser,
    LesserEqual,

    Var,
    If,
    Else,
    While,
    For,
    Return,
    Nil,
    And,
    Or,
    Print,
    Class,
    This,
    Super,

    Identifier,
    String,
    Number,
    True,
    False,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    token_type: TokenType,
    line: usize,
    lexeme: String,
    literal: Option<Literal>,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        line: usize,
        lexeme: String,
        literal: Option<Literal>,
    ) -> Self {
        Token {
            token_type,
            line,
            lexeme,
            literal,
        }
    }
}
