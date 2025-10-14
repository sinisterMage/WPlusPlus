#ifdef __cplusplus
extern "C" {
#endif

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

    if (addr < 0x1000) return 0;
    if (addr > 0x7fffffffffff) return 0;

    size_t count = 0;
    while (count < 1024) {
        unsigned char ch = s[count];
        if (ch == 0) return 1;
        if ((ch < 9 && ch != '\n' && ch != '\r') || ch > 126)
            return 0;
        count++;
    }
    return 1;
}

// =====================================================
// === SAFE PRINT
// =====================================================
static int is_probably_valid_utf8(const unsigned char *s) {
    if (!s) return 0;
    uintptr_t addr = (uintptr_t)s;

    if (addr < 0x1000) return 0;
    if (addr > 0x7fffffffffff) return 0;

    size_t i = 0;
    while (i < 1024) {
        unsigned char c = s[i];
        if (c == 0) return 1; // null terminator found

        // Single-byte (ASCII)
        if (c < 0x80) { i++; continue; }

        // Multi-byte UTF-8 checks
        if ((c & 0xE0) == 0xC0) { // 2-byte sequence
            if ((s[i+1] & 0xC0) != 0x80) return 0;
            i += 2; continue;
        }
        if ((c & 0xF0) == 0xE0) { // 3-byte sequence
            if ((s[i+1] & 0xC0) != 0x80 || (s[i+2] & 0xC0) != 0x80) return 0;
            i += 3; continue;
        }
        if ((c & 0xF8) == 0xF0) { // 4-byte sequence
            if ((s[i+1] & 0xC0) != 0x80 || (s[i+2] & 0xC0) != 0x80 || (s[i+3] & 0xC0) != 0x80) return 0;
            i += 4; continue;
        }

        // Invalid byte
        return 0;
    }

    return 1;
}

static void safe_print_string_checked(const char *s) {
    if (!s) {
        printf("(null)\n");
        return;
    }

    if (!is_probably_valid_utf8((const unsigned char *)s)) {
        printf("(invalid UTF-8 or ptr=%p)\n", s);
        return;
    }

    size_t len = strlen(s);
    if (len > 300) {
        fwrite(s, 1, 300, stdout);
        printf("... [truncated %zu bytes]\n", len - 300);
    } else {
        fwrite(s, 1, len, stdout);
        fputc('\n', stdout);
    }
}

// =====================================================
// === ARRAY / OBJECT PRINTERS (EXPORTED)
// =====================================================
__attribute__((visibility("default")))
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

__attribute__((visibility("default")))
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
// === UNIFIED VALUE PRINTER (EXPORTED)
// =====================================================
__attribute__((visibility("default")))
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

// =====================================================
// === INT PRINTER (EXPORTED)
// =====================================================
__attribute__((visibility("default")))
void wpp_print_i32(int32_t value) {
    printf("%d\n", value);
}

#ifdef __cplusplus
}
#endif
