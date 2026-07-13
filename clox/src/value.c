#include "../include/value.h"
#include "../include/memory.h"
#include <stdio.h>

void initValueArr(ValueArray *arr) {
    arr->count = 0;
    arr->capacity = 0;
    arr->values = NULL;
}

void writeValueArr(ValueArray *arr, Value v) {
    if (arr->capacity < arr->count + 1) {
        int oldCapacity = arr->capacity;
        arr->capacity = GROW_CAPACITY(oldCapacity);
        arr->values =
            GROW_ARRAY(Value, arr->values, oldCapacity, arr->capacity);
    }

    arr->values[arr->count] = v;
    arr->count++;
}

void freeValueArr(ValueArray *arr) {
    FREE_ARRAY(Value, arr->values, arr->capacity);
    initValueArr(arr);
}

void printValue(Value value) {
    printf("%g", value);
}
