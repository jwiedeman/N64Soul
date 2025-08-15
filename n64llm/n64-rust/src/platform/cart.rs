use crate::io::rom_reader::RomReader;

/// Abstraction over a ROM source (flat or banked).
pub trait RomSource {
    fn read_abs(&mut self, rom_abs_off: u64, dst: &mut [u8]) -> Result<(), ()>;
}

impl<T: RomReader + ?Sized> RomSource for &mut T {
    fn read_abs(&mut self, rom_abs_off: u64, dst: &mut [u8]) -> Result<(), ()> {
        if (**self).read(rom_abs_off, dst) { Ok(()) } else { Err(()) }
    }
}
