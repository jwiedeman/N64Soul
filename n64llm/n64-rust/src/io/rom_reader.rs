use crate::{config, n64_sys};

pub trait RomReader {
    /// Reads `dst.len()` bytes from absolute ROM offset.
    /// Returns false on failure (OOB, DMA error, etc.)
    fn read(&mut self, rom_abs_off: u64, dst: &mut [u8]) -> bool;
    fn rom_limit_bytes(&self) -> u64; // for diagnostic prints
}

pub struct FlatRomReader;

impl RomReader for FlatRomReader {
    fn read(&mut self, mut off: u64, mut dst: &mut [u8]) -> bool {
        if off % config::ROM_ALIGN as u64 != 0 {
            return false;
        }
        if off + dst.len() as u64 > self.rom_limit_bytes() {
            return false;
        }
        while !dst.is_empty() {
            let chunk = core::cmp::min(dst.len(), config::BURST_BYTES);
            unsafe {
                n64_sys::pi_read(
                    dst.as_mut_ptr(),
                    (n64_sys::CART_ROM_BASE as u32).wrapping_add(off as u32),
                    chunk as u32,
                );
            }
            off += chunk as u64;
            let tmp = dst;
            dst = &mut tmp[chunk..];
        }
        true
    }

    fn rom_limit_bytes(&self) -> u64 {
        config::ROM_LIMIT_BYTES
    }
}
