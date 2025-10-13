#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <ctype.h>

// =====================================================
// === SAFE STRING VALIDATOR (Detects invalid pointers)
// =====================================================
static int is_probably_valid_string(const char *s) {
    if (!s) return 0;
    uintptr_t addr = (uintptr_t)s;

    // Check alignment and range sanity
    if (addr < 0x1000) return 0;             // null / low mem
    if (addr > 0x7fffffffffff) return 0;     // unrealistic address

    size_t count = 0;
    while (count < 1024) {
        unsigned char ch = s[count];
        if (ch == 0) return 1;               // valid null terminator
        if ((ch < 9 && ch != '\n' && ch != '\r') || ch > 126)
            return 0;                        // non-printable
        count++;
    }
    return 1; // likely valid printable string
}

// =====================================================
// === SAFE PRINT
// =====================================================
static void safe_print_string_checked(const char *s) {
    if (!s) {
        printf("(null)\n");
        return;
    }
    if (!is_probably_valid_string(s)) {
        printf("(invalid ptr=%p)\n", s);
        return;
    }
    size_t len = strlen(s);
    if (len > 300)
        printf("%.*s... [truncated %zu bytes]\n", 300, s, len - 300);
    else
        printf("%s\n", s);
}

// =====================================================
// === ARRAY / OBJECT PRINTERS (unchanged)
// =====================================================
void wpp_print_array(int32_t *arr) {
    if (!arr) { printf("(null array)\n"); return; }
    int32_t len = arr[0];
    printf("[");
    for (int i = 0; i < len; i++) {
        printf("%d", arr[i + 1]);
        if (i < len - 1) printf(", ");
    }
    printf("]\n");
}

void wpp_print_object(void *obj_ptr) {
    if (!obj_ptr) { printf("(null object)\n"); return; }
    int32_t len = *(int32_t *)obj_ptr;
    void **fields = (void **)((char *)obj_ptr + 8);
    const char **keys = (const char **)fields[0];
    int32_t *vals = (int32_t *)fields[1];
    printf("{");
    for (int i = 0; i < len; i++) {
        printf("%s: %d", keys[i], vals[i]);
        if (i < len - 1) printf(", ");
    }
    printf("}\n");
}

// =====================================================
// === UNIFIED VALUE PRINTER (diagnostic)
// =====================================================
void wpp_print_value(void *ptr, int32_t type_id) {
    printf("[C] wpp_print_value(ptr=%p, type=%d)\n", ptr, type_id);

    if (!ptr) {
        printf("(null)\n");
        return;
    }

    switch (type_id) {
        case 1:
            wpp_print_array((int32_t *)ptr);
            break;
        case 2:
            wpp_print_object(ptr);
            break;
        default:
            safe_print_string_checked((const char *)ptr);
            break;
    }
}

void wpp_print_i32(int32_t value) {
    printf("%d\n", value);
}
