#include <stdio.h>
#include <stdint.h>
#include <string.h>

// Array: [length][elements...]
void wpp_print_array(int32_t *arr) {
    int32_t len = arr[0];
    printf("[");
    for (int i = 0; i < len; i++) {
        printf("%d", arr[i + 1]);
        if (i < len - 1) printf(", ");
    }
    printf("]");
}

// Object: { length, keys[], values[] } in { i32, ptr, ptr } layout
void wpp_print_object(void *obj_ptr) {
    int32_t len = *(int32_t *)obj_ptr;
    void **fields = (void **)((char *)obj_ptr + 8); // skip i32 + ptr alignment
    const char **keys = (const char **)fields[0];
    int32_t *vals = (int32_t *)fields[1];

    printf("{");
    for (int i = 0; i < len; i++) {
        printf("%s: %d", keys[i], vals[i]);
        if (i < len - 1) printf(", ");
    }
    printf("}");
}

void wpp_print_value(void *ptr, int32_t type_id) {
    switch (type_id) {
        case 1: wpp_print_array((int32_t *)ptr); break;
        case 2: wpp_print_object(ptr); break;
        default: printf("<unknown>"); break;
    }
}
