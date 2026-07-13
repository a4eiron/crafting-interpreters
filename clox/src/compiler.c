#include "../include/compiler.h"
#include "../include/scanner.h"

#ifdef DEBUG_PRINT_CODE
#include "../include/debug.h"
#endif

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static void unary();
static void binary();
static void grouping();
static void number();

typedef struct {
    Token current;
    Token previous;
    bool hadError;
    bool panicMode;
} Parser;

typedef enum {
    PREC_NONE,
    PREC_ASSIGNMENT,
    PREC_OR,
    PREC_AND,
    PREC_EQUALITY,
    PREC_COMPARISION,
    PREC_TERM,
    PREC_FACTOR,
    PREC_UNARY,
    PREC_CALL,
    PREC_PRIMARY,
} Precedence;

typedef void (*ParseFn)();

typedef struct {
    ParseFn prefix;
    ParseFn infix;
    Precedence precedence;
} ParseRule;

ParseRule rules[] = {
    [TOKEN_LPAREN] = {grouping, NULL, PREC_NONE},
    [TOKEN_RPAREN] = {NULL, NULL, PREC_NONE},
    [TOKEN_LBRACE] = {NULL, NULL, PREC_NONE},
    [TOKEN_RBRACE] = {NULL, NULL, PREC_NONE},
    [TOKEN_COMMA] = {NULL, NULL, PREC_NONE},
    [TOKEN_DOT] = {NULL, NULL, PREC_NONE},
    [TOKEN_MINUS] = {unary, binary, PREC_TERM},
    [TOKEN_PLUS] = {NULL, binary, PREC_TERM},
    [TOKEN_SEMICOLON] = {NULL, NULL, PREC_NONE},
    [TOKEN_SLASH] = {NULL, binary, PREC_FACTOR},
    [TOKEN_STAR] = {NULL, binary, PREC_FACTOR},
    [TOKEN_BANG] = {NULL, NULL, PREC_NONE},
    [TOKEN_BANG_EQUAL] = {NULL, NULL, PREC_NONE},
    [TOKEN_EQUAL] = {NULL, NULL, PREC_NONE},
    [TOKEN_EQUAL_EQUAL] = {NULL, NULL, PREC_NONE},
    [TOKEN_GREATER] = {NULL, NULL, PREC_NONE},
    [TOKEN_GREATER_EQUAL] = {NULL, NULL, PREC_NONE},
    [TOKEN_LESSER] = {NULL, NULL, PREC_NONE},
    [TOKEN_LESSER_EQUAL] = {NULL, NULL, PREC_NONE},
    [TOKEN_IDENTIFIER] = {NULL, NULL, PREC_NONE},
    [TOKEN_STRING] = {NULL, NULL, PREC_NONE},
    [TOKEN_NUMBER] = {number, NULL, PREC_NONE},
    [TOKEN_AND] = {NULL, NULL, PREC_NONE},
    [TOKEN_CLASS] = {NULL, NULL, PREC_NONE},
    [TOKEN_ELSE] = {NULL, NULL, PREC_NONE},
    [TOKEN_FALSE] = {NULL, NULL, PREC_NONE},
    [TOKEN_FOR] = {NULL, NULL, PREC_NONE},
    [TOKEN_FUNC] = {NULL, NULL, PREC_NONE},
    [TOKEN_IF] = {NULL, NULL, PREC_NONE},
    [TOKEN_NIL] = {NULL, NULL, PREC_NONE},
    [TOKEN_OR] = {NULL, NULL, PREC_NONE},
    [TOKEN_PRINT] = {NULL, NULL, PREC_NONE},
    [TOKEN_RETURN] = {NULL, NULL, PREC_NONE},
    [TOKEN_SUPER] = {NULL, NULL, PREC_NONE},
    [TOKEN_THIS] = {NULL, NULL, PREC_NONE},
    [TOKEN_TRUE] = {NULL, NULL, PREC_NONE},
    [TOKEN_VAR] = {NULL, NULL, PREC_NONE},
    [TOKEN_WHILE] = {NULL, NULL, PREC_NONE},
    [TOKEN_ERROR] = {NULL, NULL, PREC_NONE},
    [TOKEN_EOF] = {NULL, NULL, PREC_NONE},
};

static ParseRule *getParseRule(TokenType type) {
    return &rules[type];
}

Parser parser;
Chunk *compilingChunk;

static Chunk *currentChunk() {
    return compilingChunk;
}

///////////////////////////////////////////////////////////////////////
static void errAt(Token *token, const char *msg) {
    if (parser.panicMode)
        return;

    parser.panicMode = true;
    fprintf(stderr, "line: %d | ", token->line);

    if (token->type == TOKEN_EOF) {
        fprintf(stderr, " at end");
    } else if (token->type == TOKEN_ERROR) {
    } else {
        fprintf(stderr, " at %*.s", token->length, token->start);
    }

    fprintf(stderr, " : %s", msg);
    parser.hadError = true;
}

static void errAtCurrent(const char *msg) {
    errAt(&parser.current, msg);
}

static void err(const char *msg) {
    errAt(&parser.previous, msg);
}

///////////////////////////////////////////////////////////////////////
static void advance() {
    parser.previous = parser.current;
    for (;;) {
        parser.current = scanToken();
        if (parser.current.type != TOKEN_ERROR)
            break;

        errAtCurrent(parser.current.start);
    }
}

static void consume(TokenType type, const char *msg) {
    if (parser.current.type == type) {
        advance();
        return;
    }
    errAtCurrent(msg);
}

static void emitByte(uint8_t byte) {
    writeChunk(currentChunk(), byte, parser.previous.line);
}

static void emitBytes(uint8_t byte1, uint8_t byte2) {
    emitByte(byte1);
    emitByte(byte2);
}

static void emitReturn() {
    emitByte(OP_RETURN);
}

//////////////////////////////////////////////////////////////
static uint8_t makeConstant(Value value) {
    int constant = addConstant(currentChunk(), value);
    if (constant > UINT8_MAX) {
        err("too many constants in one chunk");
        return 0;
    }
    return (uint8_t)constant;
}

static void emitConstant(Value value) {
    emitBytes(OP_CONSTANT, makeConstant(value));
}

static void number() {
    double value = strtod(parser.previous.start, NULL);
    emitConstant(value);
}

//////////////////////////////////////////////////////////////
static void parsePredence(Precedence prec) {
    advance();
    ParseFn prefixRule = getParseRule(parser.previous.type)->prefix;
    if (prefixRule == NULL) {
        err("expected expression");
        return;
    }
    prefixRule();

    while (prec <= getParseRule(parser.current.type)->precedence) {
        advance();
        ParseFn infixRule = getParseRule(parser.previous.type)->infix;
        infixRule();
    }
}

///////////////////////////////////////////////////////////////
static void expression() {
    parsePredence(PREC_ASSIGNMENT);
}

//////////////////////////////////////////////////////////////
static void grouping() {
    expression();
    consume(TOKEN_RPAREN, "expect ')' after expression");
}

//////////////////////////////////////////////////////////////
static void unary() {
    TokenType type = parser.previous.type;

    parsePredence(PREC_UNARY);

    switch (type) {
    case TOKEN_MINUS:
        emitByte(OP_NEGATE);
        break;
    default:
        return;
    }
}

//////////////////////////////////////////////////////////////
static void binary() {
    TokenType opType = parser.previous.type;
    ParseRule *rule = getParseRule(opType);
    parsePredence((Precedence)(rule->precedence + 1));

    switch (opType) {
    case TOKEN_PLUS:
        emitByte(OP_ADD);
        break;
    case TOKEN_MINUS:
        emitByte(OP_SUBTRACT);
        break;
    case TOKEN_STAR:
        emitByte(OP_MULTIPLY);
        break;
    case TOKEN_SLASH:
        emitByte(OP_DIVIDE);
        break;
    default:
        return;
    }
}

//////////////////////////////////////////////////////////////
static void endCompiler() {
    emitReturn();

#ifdef DEBUG_PRINT_CODE
    if (!parser.hadError) {
        disassembleChunk(currentChunk(), "code");
    }
#endif
}

bool compile(const char *source, Chunk *chunk) {
    initScanner(source);
    compilingChunk = chunk;

    parser.hadError = false;
    parser.panicMode = false;

    advance();
    expression();
    consume(TOKEN_EOF, "expect end of expression");
    endCompiler();
    return !parser.hadError;
}
