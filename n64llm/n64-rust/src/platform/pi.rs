pub const CART_BASE: u64 = 0x1000_0000; // N64 cart PI bus base

#[derive(Debug, Copy, Clone)]
pub enum PiError {
    DmaFailed,
    Misaligned,
    Oob,
}

/// Blocking PI DMA read of len = dst.len() bytes from cart-space offset.
/// Implement using your libultra bindings or low-level PI regs.
/// Must tolerate small reads (64 B) and align or split as needed.
pub fn pi_dma_read(_rom_abs_off: u64, _dst: &mut [u8]) -> Result<(), PiError> {
    // TODO: platform-specific implementation
    // Strategy:
    // - align to cart requirement (e.g., 64 B), do head/tail copies via a small scratch
    // - split into safe bursts (e.g., 32 KiB)
    // - wait for DMA complete per burst
    Err(PiError::DmaFailed) // placeholder until implemented
}
