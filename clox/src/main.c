#include "../include/chunk.h"
#include "../include/debug.h"
#include "../include/vm.h"

int main(int argc, const char *argv[]) {

    initVM();
    Chunk chunk;

    initChunk(&chunk);
    int constant = addConstant(&chunk, 1);
    writeChunk(&chunk, OP_CONSTANT, 123);
    writeChunk(&chunk, constant, 123);

    constant = addConstant(&chunk, 2);
    writeChunk(&chunk, OP_CONSTANT, 123);
    writeChunk(&chunk, constant, 123);

    writeChunk(&chunk, OP_ADD, 123);

    constant = addConstant(&chunk, 3);
    writeChunk(&chunk, OP_CONSTANT, 123);
    writeChunk(&chunk, constant, 123);

    writeChunk(&chunk, OP_DIVIDE, 123);
    writeChunk(&chunk, OP_NEGATE, 123);

    constant = addConstant(&chunk, 9999);
    writeChunk(&chunk, OP_CONSTANT, 123);
    writeChunk(&chunk, constant, 123);

    writeChunk(&chunk, OP_MULTIPLY, 123);
    writeChunk(&chunk, OP_NEGATE, 123);

    disassembleChunk(&chunk, "test chunk");
    writeChunk(&chunk, OP_RETURN, 123);
    interpret(&chunk);
    freeVM();
    freeChunk(&chunk);

    return 0;
}
