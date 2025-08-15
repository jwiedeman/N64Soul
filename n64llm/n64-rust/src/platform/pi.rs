use core::ptr::{read_volatile, write_volatile, copy_nonoverlapping};
use crate::n64_sys::{
    PI_DRAM_ADDR_REG, PI_CART_ADDR_REG, PI_RD_LEN_REG, PI_STATUS_REG,
    PI_STATUS_DMA_BUSY,
};

pub const CART_BASE: u64 = 0x1000_0000; // N64 cart PI bus base

#[derive(Debug, Copy, Clone)]
pub enum PiError {
    DmaFailed,
    Misaligned,
    Oob,
}

#[inline(always)]
fn pi_busy() -> bool {
    unsafe { (read_volatile(PI_STATUS_REG as *const u32) & PI_STATUS_DMA_BUSY) != 0 }
}

/// Blocking PI DMA read of len = dst.len() bytes from cart-space offset.
/// Splits into bursts and handles misalignment with a scratch buffer.
pub fn pi_dma_read(rom_abs_off: u64, dst: &mut [u8]) -> Result<(), PiError> {
    if dst.is_empty() { return Ok(()); }

    let mut cart = (CART_BASE + rom_abs_off) as u32;
    let mut rem = dst.len();
    let mut p = dst.as_mut_ptr();

    const BURST_MAX: usize = 32 * 1024;
    const ALIGN: usize = 64;
    static mut SCRATCH: [u8; ALIGN] = [0; ALIGN];

    unsafe {
        // Head misalignment
        let head_off = (cart as usize) & (ALIGN - 1);
        if head_off != 0 {
            let read_addr = cart - head_off as u32;
            while pi_busy() {}
            write_volatile(PI_DRAM_ADDR_REG as *mut u32, SCRATCH.as_mut_ptr() as u32);
            write_volatile(PI_CART_ADDR_REG as *mut u32, read_addr);
            write_volatile(PI_RD_LEN_REG as *mut u32, (ALIGN as u32) - 1);
            while pi_busy() {}
            let copy_sz = core::cmp::min(ALIGN - head_off, rem);
            copy_nonoverlapping(SCRATCH.as_ptr().add(head_off), p, copy_sz);
            p = p.add(copy_sz);
            cart = cart.wrapping_add(copy_sz as u32);
            rem -= copy_sz;
        }

        // Aligned bursts
        while rem >= ALIGN {
            let sz = core::cmp::min(rem & !(ALIGN - 1), BURST_MAX);
            while pi_busy() {}
            write_volatile(PI_DRAM_ADDR_REG as *mut u32, p as u32);
            write_volatile(PI_CART_ADDR_REG as *mut u32, cart);
            write_volatile(PI_RD_LEN_REG as *mut u32, (sz as u32) - 1);
            while pi_busy() {}
            p = p.add(sz);
            cart = cart.wrapping_add(sz as u32);
            rem -= sz;
        }

        // Tail
        if rem > 0 {
            while pi_busy() {}
            write_volatile(PI_DRAM_ADDR_REG as *mut u32, SCRATCH.as_mut_ptr() as u32);
            write_volatile(PI_CART_ADDR_REG as *mut u32, cart);
            write_volatile(PI_RD_LEN_REG as *mut u32, (ALIGN as u32) - 1);
            while pi_busy() {}
            copy_nonoverlapping(SCRATCH.as_ptr(), p, rem);
        }
    }
    Ok(())
}
