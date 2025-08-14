pub trait RomReader {
    /// Read dst.len() bytes from absolute ROM offset (0-based within cart space).
    /// Returns true on success.
    fn read(&mut self, rom_abs_off: u64, dst: &mut [u8]) -> bool;

    /// Maximum safe bytes from RomReader's perspective (diagnostics only).
    fn rom_limit_bytes(&self) -> u64 { u64::MAX }
}

/// Flat "ROM-like" mapping over the PI bus.
/// The platform module provides `pi_dma_read(off, dst)` that blocks until done.
pub struct FlatRomReader;

impl FlatRomReader {
    pub const fn new() -> Self { Self }
}

impl RomReader for FlatRomReader {
    fn read(&mut self, rom_abs_off: u64, dst: &mut [u8]) -> bool {
        crate::platform::pi::pi_dma_read(rom_abs_off, dst)
            .map(|_| ()).is_ok()
    }
}

