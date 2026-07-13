#ifndef CLOX_SCANNER_H
#define CLOX_SCANNER_H

typedef struct {
    const char *start;
    const char *current;
    int line;
} Scanner;

typedef enum {
    TOKEN_LPAREN,
    TOKEN_RPAREN,
    TOKEN_LBRACE,
    TOKEN_RBRACE,
    TOKEN_COMMA,
    TOKEN_DOT,
    TOKEN_PLUS,
    TOKEN_MINUS,
    TOKEN_STAR,
    TOKEN_SLASH,
    TOKEN_SEMICOLON,
    TOKEN_COLON,
    TOKEN_EQUAL,
    TOKEN_BANG,
    TOKEN_BANG_EQUAL,
    TOKEN_EQUAL_EQUAL,
    TOKEN_LESSER,
    TOKEN_LESSER_EQUAL,
    TOKEN_GREATER,
    TOKEN_GREATER_EQUAL,
    TOKEN_IDENTIFIER,
    TOKEN_STRING,
    TOKEN_NUMBER,
    TOKEN_AND,
    TOKEN_OR,
    TOKEN_FUNC,
    TOKEN_CLASS,
    TOKEN_SUPER,
    TOKEN_THIS,
    TOKEN_WHILE,
    TOKEN_FOR,
    TOKEN_TRUE,
    TOKEN_FALSE,
    TOKEN_IF,
    TOKEN_ELSE,
    TOKEN_NIL,
    TOKEN_PRINT,
    TOKEN_BREAK,
    TOKEN_CONTINUE,
    TOKEN_RETURN,
    TOKEN_VAR,
    TOKEN_EOF,
    TOKEN_ERROR
} TokenType;

typedef struct {
    TokenType type;
    const char *start;
    int length;
    int line;
} Token;

void initScanner(const char *source);
Token scanToken();
#endif
