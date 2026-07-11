use std::fmt;

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
    Question,
    Colon,

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
    Break,
    Continue,
    Func,

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
    True,
    False,
    Nil,
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

    pub fn token_type(&self) -> TokenType {
        self.token_type
    }

    pub fn literal(&self) -> Option<&Literal> {
        self.literal.as_ref()
    }
    pub fn line(&self) -> usize {
        self.line
    }

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            TokenType::LParen => "(",
            TokenType::RParen => ")",
            TokenType::LBrace => "{",
            TokenType::RBrace => "}",
            TokenType::Plus => "+",
            TokenType::Minus => "-",
            TokenType::Star => "*",
            TokenType::Slash => "/",
            TokenType::Dot => ".",
            TokenType::Comma => ",",
            TokenType::Semicolon => ";",
            TokenType::Question => "?",
            TokenType::Colon => ":",

            TokenType::Bang => "!",
            TokenType::BangEqual => "!=",
            TokenType::Equal => "=",
            TokenType::EqualEqual => "==",
            TokenType::Greater => ">",
            TokenType::GreaterEqual => ">=",
            TokenType::Lesser => "<",
            TokenType::LesserEqual => "<=",

            TokenType::Var => "var",
            TokenType::If => "if",
            TokenType::Else => "else",
            TokenType::While => "while",
            TokenType::For => "for",
            TokenType::Return => "return",
            TokenType::Nil => "nil",
            TokenType::And => "and",
            TokenType::Or => "or",
            TokenType::Print => "print",
            TokenType::Class => "class",
            TokenType::This => "this",
            TokenType::Super => "super",
            TokenType::Break => "break",
            TokenType::Continue => "continue",
            TokenType::Func => "func",

            TokenType::Identifier => "identifier",
            TokenType::String => "string",
            TokenType::Number => "number",
            TokenType::True => "true",
            TokenType::False => "false",
            TokenType::Eof => "EOF",
        };

        write!(f, "{text}")
    }
}
