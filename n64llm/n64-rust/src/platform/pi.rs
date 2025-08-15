use crate::config::{BURST_BYTES, ROM_LIMIT_BYTES};
use crate::n64_sys::pi_read;

pub const CART_BASE: u64 = 0x1000_0000; // N64 cart PI bus base

#[derive(Debug, Copy, Clone)]
pub enum PiError {
    DmaFailed,
    Misaligned,
    Oob,
}

/// Uses low-level PI to DMA chunks directly into RDRAM.
pub fn pi_dma_read(rom_abs_off: u64, dst: &mut [u8]) -> Result<(), PiError> {
    // Reject ranges beyond configured ROM window.
    let end = rom_abs_off
        .checked_add(dst.len() as u64)
        .ok_or(PiError::Oob)?;
    if end > ROM_LIMIT_BYTES {
        return Err(PiError::Oob);
    }

    // Transfer in bursts; pi_read waits for DMA completion internally.
    let mut done = 0usize;
    while done < dst.len() {
        let chunk = core::cmp::min(dst.len() - done, BURST_BYTES);
        let rom_addr = (CART_BASE + rom_abs_off + done as u64) as u32;
        unsafe {
            pi_read(dst.as_mut_ptr().add(done), rom_addr, chunk as u32);
        }
        done += chunk;
    }

    Ok(())
}
