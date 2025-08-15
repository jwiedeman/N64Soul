/* n64.ld – Working N64 Linker Script for a 4MB Base Console */

/* Set the entry point. Your Rust code should define _start. */
ENTRY(_start)

MEMORY
{
    /* ROM region: model and code stored on cart */
    ROM (rx)  : ORIGIN = 0x10000000, LENGTH = 1024M

    /* RDRAM region: allow for expansion pak */
    RDRAM (rwx) : ORIGIN = 0x80000000, LENGTH = 8M
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

    /* Manifest and weights sections */
    __manifest_rom_start = .;
    .model_manifest ALIGN(64) :
    {
      KEEP(*(.model_manifest*))
    } > ROM
    __manifest_rom_end = .;

    __weights_rom_start = .;
    .model_weights ALIGN(64) :
    {
         KEEP(*(.model_weights*))
    } > ROM
    __weights_rom_end = .;

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
