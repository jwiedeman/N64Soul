#include <stddef.h>
#include <string.h>

// Minimal stub implementations
int printf(const char *format, ...) { return 0; }
int puts(const char *s) { return 0; }
int fprintf(void *stream, const char *format, ...) { return 0; }
int fputc(int c, void *stream) { return 0; }
void _exit(int status) { while(1) {} }
int isatty(int fd) { return 0; }
void *sbrk(int increment) { return (void*)-1; }
void _flush_cache(void) {}

// Memory-related functions
void *memset(void *s, int c, size_t n) {
    unsigned char *p = s;
    while (n--) *p++ = (unsigned char)c;
    return s;
}

int memcmp(const void *s1, const void *s2, size_t n) {
    const unsigned char *p1 = s1, *p2 = s2;
    while (n--) {
        if (*p1 != *p2) return *p1 - *p2;
        p1++, p2++;
    }
    return 0;
}

void *memcpy(void *dest, const void *src, size_t n) {
    unsigned char *d = dest;
    const unsigned char *s = src;
    while (n--) *d++ = *s++;
    return dest;
}

// Required stubs
int getpid() { return 1; }
void hook_stdio_calls() {}
int fstat(int fd, void *buf) { return -1; }
int kill(int pid, int sig) { return -1; }
void __assert_func(const char *file, int line, const char *func, const char *expr) {}

void *__EH_FRAME_BEGIN__ = 0;
void *__CTOR_LIST__ = 0;
void *__CTOR_END__ = 0;
void *__text_start = 0;
void *__text_end = 0;
void *__assert_func_ptr = 0;
