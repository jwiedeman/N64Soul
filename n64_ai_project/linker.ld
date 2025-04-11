ENTRY(_start)

MEMORY
{
    ROM (rx)  : ORIGIN = 0xB0000000, LENGTH = 4M   /* ROM */
    RAM (rwx) : ORIGIN = 0x80000400, LENGTH = 8M   /* 8MB RAM */
}

/* Global Pointer for small-data optimization */
_gp = 0x80010000;

/* Stack Setup */
__stack_top = ORIGIN(RAM) + LENGTH(RAM) - 0x2000;  /* 8KB stack size */

SECTIONS {
    /* Code */
    .text : ALIGN(16) {
        KEEP(*(.text.start))
        *(.text)
        *(.text.*)
    } > RAM AT>ROM

    /* Read-only Data */
    .rodata : ALIGN(16) {
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

    /* Stack (Fixed 8KB) */
    .stack (NOLOAD) : ALIGN(16) {
        __stack_bottom = .;
        . = __stack_top - 0x2000;  /* Reduce stack to 8KB */
        __stack_top = .;
    } > RAM
}
