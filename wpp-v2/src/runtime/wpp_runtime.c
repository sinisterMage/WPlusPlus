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
        printf("(null)");
        return;
    }

    if (!is_probably_valid_utf8((const unsigned char *)s)) {
        printf("(invalid UTF-8 or ptr=%p)", s);
        return;
    }

    size_t len = strlen(s);
    if (len > 300) {
        fwrite(s, 1, 300, stdout);
        printf("... [truncated %zu bytes]", len - 300);
    } else {
        fwrite(s, 1, len, stdout);
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
// === UNIFIED BASIC TYPE PRINTER (EXPORT)
// =====================================================
//  type_id mapping:
//  1 = i32
//  2 = i64
//  3 = f32
//  4 = f64
//  5 = bool
//  6 = string
// =====================================================

__attribute__((visibility("default")))
void wpp_print_value_basic(const void *ptr, int32_t type_id) {
    if (!ptr) {
        printf("(null) ");
        return;
    }

    switch (type_id) {
        case 1: { // i32
            int32_t v = *(int32_t *)ptr;
            printf("%d ", v);
            break;
        }
        case 2: { // i64
            int64_t v = *(int64_t *)ptr;
            printf("%lld ", (long long)v);
            break;
        }
        case 3: { // f32
            float v = *(float *)ptr;
            printf("%.6g ", v);
            break;
        }
        case 4: { // f64
            double v = *(double *)ptr;
            printf("%.6g ", v);
            break;
        }
        case 5: { // bool
            int32_t v = *(int32_t *)ptr;
            printf("%s ", v ? "true" : "false");
            break;
        }
        case 6: { // string
            const char *s = (const char *)ptr;
            safe_print_string_checked(s);
            printf(" ");
            break;
        }

        default:
            printf("(unknown type_id=%d, ptr=%p) ", type_id, ptr);
            break;
    }
}

// =====================================================
// === READLINE (EXPORTED)
// =====================================================
__attribute__((visibility("default")))
char* wpp_readline() {
    static char buffer[1024]; // static so pointer stays valid after return
    if (fgets(buffer, sizeof(buffer), stdin) != NULL) {
        size_t len = strlen(buffer);
        if (len > 0 && buffer[len - 1] == '\n')
            buffer[len - 1] = '\0';
        return buffer;
    }
    return "";
}

// =====================================================
// === INTEGER TO STRING CONVERSION
// =====================================================
__attribute__((visibility("default")))
char* wpp_int_to_string(int32_t value) {
    // Thread-local buffer to avoid conflicts
    static _Thread_local char buffer[32];
    snprintf(buffer, sizeof(buffer), "%d", value);
    return buffer;
}

#ifdef __cplusplus
}
#endif
