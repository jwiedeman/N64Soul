#ifndef RUNTIME_H
#define RUNTIME_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
    uint8_t *base;
    size_t offset;
    size_t size;
} arena_t;

static inline void arena_init(arena_t *a, void *mem, size_t size) {
    a->base = mem;
    a->offset = 0;
    a->size = size;
}

static inline void *arena_alloc(arena_t *a, size_t size) {
    size_t aligned = (size + 15) & ~((size_t)15);
    if (a->offset + aligned > a->size) return NULL;
    void *p = a->base + a->offset;
    a->offset += aligned;
    return p;
}

#define CANARY_VALUE 0xDEADBEEF
#define CANARY_DECL(name) uint32_t name = CANARY_VALUE
#define CANARY_CHECK(name) ((name) == CANARY_VALUE)

void watchdog_tick(unsigned ms);
#define WATCHDOG_TICK(ms) watchdog_tick(ms)

extern uint8_t __dmabuf_start[];
extern uint8_t __dmabuf_end[];
extern uint8_t __weights_page0[];
extern uint8_t __weights_page1[];

#endif /* RUNTIME_H */
