#include <stddef.h>
#include <stdint.h>
#include <string.h>

void cache_invalidate(void *addr, size_t len) {
    (void)addr; (void)len; /* Stub for cache invalidate */
}

void cache_writeback(void *addr, size_t len) {
    (void)addr; (void)len; /* Stub for cache writeback */
}

void dma_fetch_page(void *dst, uint32_t rom_off, size_t len) {
    (void)rom_off; /* ROM offset ignored in stub */
    cache_invalidate(dst, len);
    memset(dst, 0, len); /* Stub DMA: zero buffer */
    cache_writeback(dst, len);
}

