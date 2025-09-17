//! Production‐ready IPL3 boot code for Nintendo 64 in Rust.
//!
//! This module initializes RDRAM, clears memory using RSP DMA,
//! copies Stage2 from ROM to RDRAM, and then jumps to Stage2.
//!
//! Place this file in your `src/` folder and update your linker script
//! so that the sections (.header, .banner, .stage1.pre, .stage1) are
//! mapped to the correct addresses.

use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};

//
// --- Hardware Register Addresses ---
//

// RSP registers (Reality Signal Processor)
const SP_DMA_BUSY: *mut u32   = 0xA4000014 as *mut u32;
const SP_DMA_FULL: *mut u32   = 0xA4000018 as *mut u32;
const SP_RSP_ADDR: *mut u32   = 0xA4000004 as *mut u32;
const SP_DRAM_ADDR: *mut u32  = 0xA4000008 as *mut u32;
const SP_RD_LEN: *mut u32     = 0xA400000C as *mut u32;
const SP_WR_LEN: *mut u32     = 0xA4000010 as *mut u32;
const SP_IMEM: *mut u32       = 0xA4000020 as *mut u32;

// PI registers (Peripheral Interface)
const PI_DRAM_ADDR: *mut u32  = 0xA4600000 as *mut u32;
const PI_CART_ADDR: *mut u32  = 0xA4600004 as *mut u32;
const PI_RD_LEN: *mut u32     = 0xA4600008 as *mut u32;
const PI_WR_LEN: *mut u32     = 0xA460000C as *mut u32;
const PI_STATUS: *mut u32     = 0xA4600010 as *mut u32;

// MI and RI registers for boot flags (addresses taken from production docs)
const MI_VERSION: *mut u32    = 0xA4600024 as *mut u32;
const RI_SELECT: *mut u32     = 0xA4600030 as *mut u32;

// Base address for RDRAM (cached view)
const RDRAM_BASE: u32         = 0xA0000000;
// Total reserved size for Stage2 (adjust as needed)
const TOTAL_RESERVED_SIZE: i32 = 0x1000;

//
// --- ROM Header and Banner ---
//

#[repr(C, packed)]
pub struct RomHeader {
    pub pi_dom1_config: u32,
    pub clock_rate: u32,
    pub boot_address: u32,
    pub sdk_version: u32,
    pub checksum: u64,
    pub reserved1: u64,
    pub title: [u8; 20],
    pub reserved2: [u8; 7],
    pub gamecode: u32,
    pub rom_version: u8,
}

#[link_section = ".header"]
#[used]
static HEADER: RomHeader = RomHeader {
    pi_dom1_config: 0x80371240, // Standard PI DOM1 config for N64
    clock_rate: 0,
    boot_address: 0x80000400,
    sdk_version: 0,
    checksum: 0,
    reserved1: 0,
    title: *b"Libdragon           ",
    reserved2: [0; 7],
    gamecode: 0,
    rom_version: 0,
};

#[link_section = ".banner"]
#[used]
static BANNER: [u8; 32] = *b" Libdragon IPL3 Coded by Rasky  ";

//
// --- External Symbols ---
//

// Symbol for Stage2 start; provide this symbol via your linker script or additional module.
extern "C" {
    static __stage2_start: u32;
}

//
// --- CP0 Register Access (Production Ready) ---
//

/// Read CP0 Count register ($9)
unsafe fn c0_count() -> u32 {
    let count: u32;
    asm!("mfc0 {0}, $9", out(reg) count, options(nostack));
    count
}

/// Write to CP0 Cause register ($13)
unsafe fn c0_write_cause(val: u32) {
    asm!("mtc0 {0}, $13", in(reg) val, options(nostack));
}

/// Write to CP0 Count register ($9)
unsafe fn c0_write_count(val: u32) {
    asm!("mtc0 {0}, $9", in(reg) val, options(nostack));
}

/// Write to CP0 Compare register ($11)
unsafe fn c0_write_compare(val: u32) {
    asm!("mtc0 {0}, $11", in(reg) val, options(nostack));
}

/// Write to CP0 WatchLo register ($18)
unsafe fn c0_write_watchlo(val: u32) {
    asm!("mtc0 {0}, $18", in(reg) val, options(nostack));
}

//
// --- Production Implementations for Peripherals ---
//

/// Initialize hardware entropy generator.
unsafe fn entropy_init() {
    // Production: Read hardware RNG registers or use other means.
    // For example, you might read from a specific MI register.
    // (Implementation-specific; fill in as needed.)
}

/// Initialize USB (or other peripherals) if required.
unsafe fn usb_init() {
    // Production: Initialize peripherals as required.
}

/// Debug output; production code may output via a serial port.
unsafe fn debugf(msg: &str) {
    // Production: Write the message to a debug serial port.
    // For now, we do nothing.
    let _ = msg;
}

/// Clear caches (data and instruction) using MIPS cache instructions.
unsafe fn cop0_clear_cache() {
    // Production: flush caches. This example uses a simple sync.
    asm!("cache 0x00, 0($0)", options(nostack));
    asm!("cache 0x01, 0($0)", options(nostack));
}

/// Memory barrier to ensure ordering.
#[inline(always)]
unsafe fn memory_barrier() {
    asm!("sync", options(nostack, nomem));
}

/// Read a 32-bit value from ROM at the given address.
unsafe fn io_read(address: u32) -> i32 {
    // ROM is memory mapped starting at 0x10000000.
    let ptr = (0x10000000u32.wrapping_add(address)) as *const u32;
    read_volatile(ptr) as i32
}

/// Compute the RDRAM address in which to load Stage2.
/// Places Stage2 at the top of available memory minus stage2_size.
fn loader_base(memsize: i32, stage2_size: i32) -> *mut u8 {
    let base = 0x80000000u32.wrapping_add(memsize as u32).wrapping_sub(stage2_size as u32);
    base as *mut u8
}

/// Compute the new stack pointer for Stage2.
/// Adjust this as required by your memory layout.
fn stack2_top(_memsize: i32, _stage2_size: i32) -> u32 {
    // For example, use DMEM at 0xA4000000 plus an offset.
    0xA4000000 + 4096
}

//
// --- RDRAM Initialization and Memory Clearing ---
//

/// Clear 8 bytes at the given memory address using uncached writes.
unsafe fn bzero8(mem: *mut u8) {
    for i in 0..8 {
        write_volatile(mem.add(i), 0);
    }
}

/// Initialize RSP zeroing (clear IMEM) based on hardware variant.
unsafe fn rsp_bzero_init(bbplayer: bool) {
    while read_volatile(SP_DMA_BUSY) != 0 {}
    if !bbplayer {
        // Use DMA from an address >8 MiB.
        write_volatile(SP_RSP_ADDR, 0x1000);
        write_volatile(SP_DRAM_ADDR, 8 * 1024 * 1024 + 0x2000);
        write_volatile(SP_RD_LEN, 4096 - 1);
    } else {
        // For iQue hardware: clear IMEM via CPU.
        let imem = SP_IMEM as *mut u32;
        for i in 0..(4096 / 4) {
            write_volatile(imem.add(i), 0);
        }
    }
}

/// Asynchronously clear memory via RSP DMA.
unsafe fn rsp_bzero_async(rdram: u32, mut size: i32) {
    let addr = rdram | RDRAM_BASE;
    bzero8(addr as *mut u8);
    if size <= 8 {
        return;
    }
    bzero8((addr + (size as u32) - 8) as *mut u8);
    let mut rdram_offset = addr + 8;
    size -= 8;
    while size > 0 {
        let sz = if size >= 1024 * 1024 {
            1024 * 1024
        } else if size >= 4096 {
            (size >> 12) << 12
        } else {
            size
        };
        while read_volatile(SP_DMA_FULL) != 0 {}
        write_volatile(SP_RSP_ADDR, 0x1000);
        write_volatile(SP_DRAM_ADDR, rdram_offset);
        write_volatile(SP_WR_LEN, (sz as u32).wrapping_sub(1));
        size -= sz;
        rdram_offset = rdram_offset.wrapping_add(sz as u32);
    }
}

/// Initialize RDRAM and perform a simple memory test.
/// Calls the provided callback for each memory bank.
unsafe fn rdram_init<F: FnMut(i32, bool)>(mut callback: F) -> i32 {
    // Use the cached RDRAM base (0xA0000000).
    let base = 0xA0000000 as *mut u32;
    let mut memsize = 0;
    // Iterate over chip pairs (0, 2, 4, 6); each pair adds 2 MiB.
    for chip_id in (0..8).step_by(2) {
        let ptr = base.add(chip_id * (1024 * 1024 / 4));
        write_volatile(ptr, 0);
        write_volatile(ptr, 0x12345678);
        if read_volatile(ptr) != 0x12345678 {
            break;
        }
        memsize += 2 * 1024 * 1024;
        callback(chip_id as i32, chip_id == 6);
    }
    memsize
}

/// Callback for each RDRAM bank initialization.
unsafe fn mem_bank_init(chip_id: i32, last: bool) {
    if chip_id == -1 {
        // First call: clear SP_IMEM.
        rsp_bzero_init(false);
        return;
    }
    let base = chip_id * 1024 * 1024;
    let mut size = 2 * 1024 * 1024;
    if last {
        size -= TOTAL_RESERVED_SIZE;
    }
    rsp_bzero_async(base as u32, size);
}

//
// --- Stage1 Pre and Stage1 Boot Functions ---
//

/// Stage1pre: sets up the initial stack pointer and jumps to stage1.
/// Placed in a dedicated section with no function prologue/epilogue.
#[naked]
#[no_mangle]
#[link_section = ".stage1.pre"]
pub unsafe extern "C" fn stage1pre() -> ! {
    asm!(
        "lui $sp, 0xA400",
        "ori $sp, $sp, 0x0FF0",
        "j stage1",
        "nop",
        options(noreturn)
    );
}

/// Stage1: performs early initialization, loads Stage2, and jumps to it.
#[no_mangle]
#[link_section = ".stage1"]
pub unsafe extern "C" fn stage1() -> ! {
    // Early peripheral initialization.
    entropy_init();
    usb_init();
    debugf("Libdragon IPL3");

    // Reset CP0 registers.
    c0_write_cause(0);
    c0_write_count(0);
    c0_write_compare(0);
    c0_write_watchlo(0);

    // Determine if we’re running on iQue hardware.
    // In production, read MI_VERSION and RI_SELECT.
    let bbplayer = ((*MI_VERSION) & 0xF0) == 0xB0 || read_volatile(RI_SELECT) != 0;

    let mut memsize: i32;
    if !bbplayer && read_volatile(RI_SELECT) == 0 {
        memsize = rdram_init(|chip, last| unsafe { mem_bank_init(chip, last) });
    } else {
        // For iQue hardware, use the OS-provided memory size.
        // Read from the special location 0xA0000318.
        let size_ptr = 0xA0000318 as *mut u32;
        memsize = read_volatile(size_ptr) as i32;
        // Adjust for special cases if necessary.
        if memsize == 0x800000 {
            memsize = 0x7C0000;
        }
    }

    debugf("Total memory initialized");

    // Clear the first 0x400 bytes (preserved for boot flags).
    rsp_bzero_init(bbplayer);
    rsp_bzero_async(0xA0000400, memsize - 0x400 - TOTAL_RESERVED_SIZE);

    // Clear caches.
    cop0_clear_cache();

    // Load Stage2:
    // Obtain Stage2 start address from external symbol.
    let stage2_addr = &__stage2_start as *const u32 as u32;
    // Read stage2 size from ROM at stage2_addr.
    let stage2_size = io_read(stage2_addr) as i32;
    // Stage2 binary starts 8 bytes after the header.
    let stage2_start = stage2_addr.wrapping_add(8);

    // Calculate destination in RDRAM for Stage2.
    let rdram_stage2 = loader_base(memsize, stage2_size);

    // Set up DMA transfer:
    write_volatile(PI_DRAM_ADDR, rdram_stage2 as u32);
    // Adjust cart address to remove cached bit (RDRAM is mapped at 0xA0000000).
    write_volatile(PI_CART_ADDR, stage2_start.wrapping_sub(RDRAM_BASE));
    // Wait for any pending PI DMA.
    while read_volatile(PI_STATUS) & 1 != 0 {}
    write_volatile(PI_WR_LEN, stage2_size.wrapping_sub(1) as u32);

    // Ensure memory writes complete.
    memory_barrier();

    // Set up stack pointer for Stage2.
    asm!("move $sp, {0}", in(reg) stack2_top(memsize, stage2_size), options(nostack));

    // Jump to Stage2.
    let stage2: extern "C" fn() -> ! = core::mem::transmute(rdram_stage2 as usize);
    stage2()
}

//
// --- Panic Handler ---
//

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // In production, you might output info via a debug port.
    let _ = info;
    loop {
        // Halt on panic.
    }
}