#include "../include/value.h"
#include "../include/memory.h"
#include <stdio.h>

void initValueArr(ValueArray *arr) {
    arr->count = 0;
    arr->capactiy = 0;
    arr->values = NULL;
}

void writeValueArr(ValueArray *arr, Value v) {
    if (arr->capactiy < arr->count + 1) {
        int oldCapacity = arr->capactiy;
        arr->capactiy = GROW_CAPACITY(oldCapacity);
        arr->values =
            GROW_ARRAY(Value, arr->values, oldCapacity, arr->capactiy);
    }

    arr->values[arr->count] = v;
    arr->count++;
}

void freeValueArr(ValueArray *arr) {
    FREE_ARRAY(Value, arr->values, arr->capactiy);
    initValueArr(arr);
}

void printValue(Value value) {
    printf("%g", value);
}
