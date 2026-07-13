#include "../include/vm.h"
#include <stdio.h>
#include <stdlib.h>

static char *readFile(const char *path) {
    FILE *fp = fopen(path, "rb");

    if (fp == NULL) {
        fprintf(stderr, "couldn't open the file %s", path);
        exit(74);
    }

    fseek(fp, 0L, SEEK_END);
    size_t file_size = ftell(fp);
    rewind(fp);

    char *buffer = (char *)malloc(file_size + 1);
    if (buffer == NULL) {
        fprintf(stderr, "buy some RAM");
        exit(74);
    }

    size_t bytes_read = fread(buffer, sizeof(char), file_size, fp);
    if (bytes_read < file_size) {
        fprintf(stderr, "couldn't read the file %s", path);
        exit(74);
    }

    buffer[bytes_read] = '\0';
    fclose(fp);

    return buffer;
}

static void runFile(const char *path) {
    char *source = readFile(path);
    InterpretResult result = interpret(source);
    free(source);

    if (result == INTERPRET_COMPILE_ERROR)
        exit(65);

    if (result == INTERPRET_RUNTIME_ERROR)
        exit(70);
}

int main(int argc, const char *argv[]) {

    if (argc != 2) {
        fprintf(stderr, "Usage: clox [path]\n");
        exit(64);
    }

    initVM();
    runFile(argv[1]);
    freeVM();

    return 0;
}
