#include "../include/scanner.h"
#include "../include/common.h"
#include <ctype.h>
#include <string.h>

Scanner scanner;

void initScanner(const char *source) {
    scanner.start = source;
    scanner.current = source;
    scanner.line = 1;
}

static bool isAtEnd() {
    return *scanner.current == '\0';
}

static char advance() {
    scanner.current++;
    return scanner.current[-1];
}

static char peek() {
    return *scanner.current;
}

static char peekNext() {
    if (isAtEnd())
        return '\0';
    return scanner.current[1];
}

static bool match(char c) {
    if (isAtEnd())
        return false;

    if (*scanner.current != c)
        return false;

    scanner.current++;
    return true;
}

static void skipWhiteSpace() {
    for (;;) {
        char c = peek();
        switch (c) {
        case ' ':
        case '\t':
        case '\r':
            advance();
            break;
        case '\n':
            scanner.line++;
            advance();
            break;
        case '/': {
            if (peekNext() == '/') {
                while (peek() != '\n' && !isAtEnd()) {
                    advance();
                }
            } else {
                return;
            }
            break;
        }
        default:
            return;
        }
    }
}

static Token errorToken(const char *msg) {
    Token token;
    token.type = TOKEN_ERROR;
    token.start = msg;
    token.length = (int)strlen(msg);
    token.line = scanner.line;
    return token;
}

static Token makeToken(TokenType type) {
    Token token;
    token.type = type;
    token.start = scanner.start;
    token.length = (int)(scanner.current - scanner.start);
    token.line = scanner.line;
    return token;
}

static Token string() {
    while (peek() != '"' && !isAtEnd()) {
        if (peek() == '\n')
            scanner.line++;
        advance();
    }

    if (isAtEnd())
        errorToken("unterminated string");

    advance();
    return makeToken(TOKEN_STRING);
}

static Token number() {
    while (isdigit(peek()))
        advance();
    if (peek() == '.' && isdigit(peekNext()))
        advance();
    while (isdigit(peek()))
        advance();
    return makeToken(TOKEN_NUMBER);
}

static TokenType checkKeyword(int start, int length, const char *rest,
                              TokenType type) {
    if (scanner.current - scanner.start == start + length &&
        memcmp(scanner.start + start, rest, length) == 0) {
        return type;
    }

    return TOKEN_IDENTIFIER;
}

static TokenType identifierType() {
    switch (scanner.start[0]) {
    case 'a':
        return checkKeyword(1, 2, "nd", TOKEN_AND);
    case 'c':
        return checkKeyword(1, 4, "lass", TOKEN_CLASS);
    case 'e':
        return checkKeyword(1, 3, "lse", TOKEN_ELSE);
    case 'i':
        return checkKeyword(1, 1, "f", TOKEN_IF);
    case 'n':
        return checkKeyword(1, 2, "il", TOKEN_NIL);
    case 'o':
        return checkKeyword(1, 1, "r", TOKEN_OR);
    case 'p':
        return checkKeyword(1, 4, "rint", TOKEN_PRINT);
    case 'r':
        return checkKeyword(1, 5, "eturn", TOKEN_RETURN);
    case 's':
        return checkKeyword(1, 4, "uper", TOKEN_SUPER);
    case 'v':
        return checkKeyword(1, 2, "ar", TOKEN_VAR);
    case 'w':
        return checkKeyword(1, 4, "hile", TOKEN_WHILE);
    case 'f':
        if (scanner.current - scanner.start > 1) {
            switch (scanner.start[1]) {
            case 'a':
                return checkKeyword(2, 3, "lse", TOKEN_FALSE);
            case 'o':
                return checkKeyword(2, 1, "r", TOKEN_FOR);
            case 'u':
                return checkKeyword(2, 2, "nc", TOKEN_FUNC);
            }
        }
        break;
    case 't':
        if (scanner.current - scanner.start > 1) {
            switch (scanner.start[1]) {
            case 'r':
                return checkKeyword(2, 2, "ue", TOKEN_TRUE);
            case 'h':
                return checkKeyword(2, 2, "is", TOKEN_THIS);
            }
        }
        break;
    }
    return TOKEN_IDENTIFIER;
}

static Token identifier() {
    while (isalnum(peek()))
        advance();
    return makeToken(identifierType());
}

Token scanToken() {
    skipWhiteSpace();
    scanner.start = scanner.current;
    if (isAtEnd())
        return makeToken(TOKEN_EOF);

    char c = advance();

    if (isalpha(c))
        return identifier();

    if (isdigit(c))
        return number();

    switch (c) {
    case '(':
        return makeToken(TOKEN_LPAREN);
    case ')':
        return makeToken(TOKEN_RPAREN);
    case '{':
        return makeToken(TOKEN_LBRACE);
    case '}':
        return makeToken(TOKEN_RBRACE);
    case ';':
        return makeToken(TOKEN_SEMICOLON);
    case '.':
        return makeToken(TOKEN_DOT);
    case ',':
        return makeToken(TOKEN_COMMA);
    case '+':
        return makeToken(TOKEN_PLUS);
    case '-':
        return makeToken(TOKEN_MINUS);
    case '*':
        return makeToken(TOKEN_STAR);
    case '/':
        return makeToken(TOKEN_SLASH);
    case '!':
        return makeToken(match('=') ? TOKEN_BANG_EQUAL : TOKEN_BANG);
    case '=':
        return makeToken(match('=') ? TOKEN_EQUAL_EQUAL : TOKEN_EQUAL);
    case '<':
        return makeToken(match('=') ? TOKEN_LESSER_EQUAL : TOKEN_LESSER);
    case '>':
        return makeToken(match('=') ? TOKEN_GREATER_EQUAL : TOKEN_GREATER);
    case '"':
        return string();
    }

    return errorToken("unexpected character");
}
