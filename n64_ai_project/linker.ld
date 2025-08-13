ENTRY(_start)

MEMORY
{
    ROM (rx)  : ORIGIN = 0xB0000000, LENGTH = 4M   /* ROM */
    RAM (rwx) : ORIGIN = 0x80000400, LENGTH = 8M   /* 8MB RAM */
}

/* Global Pointer for small-data optimization */
_gp = 0x80010000;

/* Stack Setup */
__stack_top = ORIGIN(RAM) + LENGTH(RAM) - 0x4000;  /* 16KB stack */

SECTIONS {
    /* Code */
    .text : ALIGN(16) {
        KEEP(*(.text.start))
        *(.text)
        *(.text.*)
    } > RAM AT>ROM

    /* Read-only Data */
    .rodata : ALIGN(32) {
        *(.rodata)
        *(.rodata.*)
    } > RAM AT>ROM

    /* Initialized Data */
    .data : ALIGN(16) {
        _gp = .;
        *(.data)
        *(.data.*)
    } > RAM AT>ROM

    /* Uninitialized Data */
    .bss (NOLOAD) : ALIGN(16) {
        _bss_start = .;
        *(.bss)
        *(.bss.*)
        *(COMMON)
        _bss_end = .;
    } > RAM

    /* DMA-safe buffer */
    .dmabuf (NOLOAD) : ALIGN(32) {
        __dmabuf_start = .;
        . += 0x20000; /* 128KB */
        __dmabuf_end = .;
    } > RAM

    /* Weight paging buffers */
    .weights (NOLOAD) : ALIGN(32) {
        __weights_page0 = .;
        . += 0x10000; /* 64KB */
        __weights_page1 = .;
        . += 0x10000; /* 64KB */
    } > RAM

    /* Stack */
    .stack (NOLOAD) : ALIGN(16) {
        __stack_bottom = .;
        . = __stack_top - 0x4000;  /* 16KB stack */
        __stack_top = .;
    } > RAM
}
