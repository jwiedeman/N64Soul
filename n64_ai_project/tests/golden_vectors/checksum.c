#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include "../../src/ai.h"
#include "../../include/n64.h"

static uint32_t checksum(const char *s) {
    uint32_t c = 0;
    while (*s) {
        c = c * 131 + (unsigned char)(*s++);
    }
    return c;
}

int main(void) {
    char out[256];
    process_ai("test", out);
    printf("checksum:%u\n", checksum(out));
    console_flush();
    return 0;
}
