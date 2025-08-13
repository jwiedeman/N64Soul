#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "ai.h"
#include "runtime.h"

static arena_t kernel_arena;
static uint8_t arena_mem[1024];

#ifdef USE_Q15
static int16_t activation(int16_t x) { return x; }
#else
static float activation(float x) { return x; }
#endif

void process_ai(const char *input, char *output) {
    arena_init(&kernel_arena, arena_mem, sizeof(arena_mem));
#ifdef USE_Q15
    int16_t acc = 0;
    for (const char *p = input; *p; ++p) {
        acc += activation((int16_t)*p);
    }
    snprintf(output, 64, "Q15:%d", acc);
#else
    float acc = 0.f;
    for (const char *p = input; *p; ++p) {
        acc += activation((float)*p);
    }
    snprintf(output, 64, "F:%d", (int)acc);
#endif
}

