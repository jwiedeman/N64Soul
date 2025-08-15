use core::ptr::{read_volatile, write_volatile};
use crate::n64_sys::{
    PI_DRAM_ADDR_REG, PI_CART_ADDR_REG, PI_RD_LEN_REG, PI_STATUS_REG,
    PI_STATUS_DMA_BUSY, CART_ROM_BASE,
};

#[inline(always)]
fn pi_busy() -> bool {
    unsafe { (read_volatile(PI_STATUS_REG as *const u32) & PI_STATUS_DMA_BUSY) != 0 }
}

#[derive(Debug, Copy, Clone)]
pub enum PiError {
    DmaFailed,
    Misaligned,
    Oob,
}

/// Kick off a PI DMA read but do NOT wait for completion.
pub unsafe fn pi_dma_start(dst: *mut u8, cart_addr: u32, len: u32) {
    // Wait if a previous DMA is in flight.
    while pi_busy() {}
    write_volatile(PI_DRAM_ADDR_REG as *mut u32, dst as u32);
    write_volatile(PI_CART_ADDR_REG as *mut u32, cart_addr);
    write_volatile(PI_RD_LEN_REG  as *mut u32, len - 1);
}

/// Wait for any outstanding PI DMA to finish.
#[inline(always)]
pub fn pi_dma_wait_idle() {
    while pi_busy() {}
}

/// Uses low-level PI to DMA chunks directly into RDRAM.
pub fn pi_dma_read(rom_abs_off: u64, dst: &mut [u8]) -> Result<(), PiError> {
    use crate::config::{ROM_LIMIT_BYTES, BURST_BYTES};
    if dst.is_empty() { return Ok(()); }
    let end = rom_abs_off
        .checked_add(dst.len() as u64)
        .ok_or(PiError::Oob)?;
    if end > ROM_LIMIT_BYTES {
        return Err(PiError::Oob);
    }

    let mut done = 0usize;
    while done < dst.len() {
        let chunk = core::cmp::min(dst.len() - done, BURST_BYTES);
        let rom_addr = (CART_ROM_BASE + rom_abs_off + done as u64) as u32;
        unsafe {
            pi_dma_start(dst.as_mut_ptr().add(done), rom_addr, chunk as u32);
            pi_dma_wait_idle();
        }
        done += chunk;
    }
    Ok(())
}
