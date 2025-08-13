#include <stdarg.h>
#include <stddef.h>
#include <stdio.h>
#include "n64.h"
#include "runtime.h"

#define LOG_BUFFER_SIZE 512

static char log_buffer[LOG_BUFFER_SIZE];
static volatile size_t log_head = 0;
static volatile size_t log_tail = 0;

static void enqueue_char(char c) {
    size_t next = (log_head + 1) % LOG_BUFFER_SIZE;
    if (next != log_tail) {
        log_buffer[log_head] = c;
        log_head = next;
    }
}

static void enqueue_string(const char *s) {
    while (*s) {
        enqueue_char(*s++);
    }
}

void console_init(void) {
    log_head = log_tail = 0;
}

void console_clear(void) {
    log_head = log_tail = 0;
}

void console_flush(void) {
    while (log_tail != log_head) {
        char c = log_buffer[log_tail];
        log_tail = (log_tail + 1) % LOG_BUFFER_SIZE;
        putchar(c);
    }
    fflush(stdout);
}

int printf(const char *fmt, ...) {
    char buf[128];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof(buf), fmt, ap);
    va_end(ap);
    enqueue_string(buf);
    return 0;
}

int puts(const char *s) {
    enqueue_string(s);
    enqueue_char('\n');
    return 0;
}

static volatile unsigned watchdog_counter = 0;

void watchdog_tick(unsigned ms) {
    watchdog_counter += ms;
}

