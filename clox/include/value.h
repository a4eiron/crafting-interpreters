#ifndef CLOX_VALUE_H
#define CLOX_VALUE_H

#include "common.h"

typedef double Value;

typedef struct {
    int count;
    int capacity;
    Value *values;
} ValueArray;

void initValueArr(ValueArray *arr);
void writeValueArr(ValueArray *arr, Value value);
void freeValueArr(ValueArray *arr);
void printValue(Value value);

#endif
