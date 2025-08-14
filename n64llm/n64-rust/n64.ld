/* n64.ld – Working N64 Linker Script for a 4MB Base Console */

/* Set the entry point. Your Rust code should define _start. */
ENTRY(_start)

MEMORY
{
    /* ROM region: This is where your code, constants, and model weights reside.
       Adjust LENGTH to match your cartridge size. Here we assume ~480 MB. */
    ROM (rx)  : ORIGIN = 0x10000000, LENGTH = 480M

    /* RDRAM region: Base N64 has 4 MB of RAM.
       All writable data (.data, .bss, heap, stack) live here. */
    RDRAM (rwx) : ORIGIN = 0x80000000, LENGTH = 4M
}

SECTIONS
{
    /* .text: All executable code goes into ROM */
    .text : ALIGN(4)
    {
        *(.text*)
    } > ROM

    /* .rodata: Read-only data (constants, string literals, etc.) */
    .rodata : ALIGN(4)
    {
        *(.rodata*)
    } > ROM

    /* .model_weights: Dedicated section for model weights. */
    __model_weights_rom_start = .;
    .model_weights ALIGN(64) :
    {
         KEEP(*(.model_weights*))
    } > ROM
    __model_weights_rom_end = .;
    __model_weights_rom_size = __model_weights_rom_end - __model_weights_rom_start;

    /* .data: Initialized mutable data goes into RDRAM */
    .data : ALIGN(4)
    {
        *(.data*)
    } > RDRAM

    /* .bss: Uninitialized data goes into RDRAM */
    .bss : ALIGN(4)
    {
        __bss_start = .;
        *(.bss*)
        __bss_end = .;
    } > RDRAM

    /* .stack: Reserve a block for the stack in RDRAM.
       Here we reserve 64 KB for the stack. Adjust as needed. */
    .stack (NOLOAD) : ALIGN(4)
    {
        __stack_start = .;
        . = . + 0x10000;  /* 64KB stack */
        __stack_end = .;
    } > RDRAM
}
