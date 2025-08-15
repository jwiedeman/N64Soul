// Host-only ROM adapter for tests/dev: reads from an in-memory Vec<u8>.
#[cfg(any(test, feature = "host"))]
extern crate alloc;
#[cfg(any(test, feature = "host"))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "host"))]
pub struct VecRom(pub Vec<u8>);

#[cfg(any(test, feature = "host"))]
impl crate::platform::cart::RomSource for VecRom {
    fn read_abs(&mut self, off: u64, dst: &mut [u8]) -> Result<(), ()> {
        let off = off as usize;
        let end = off + dst.len();
        let src = &self.0[off..end];
        dst.copy_from_slice(src);
        Ok(())
    }
}

#[cfg(any(test, feature = "host"))]
impl VecRom {
    pub fn size_bytes(&self) -> Option<u64> { Some(self.0.len() as u64) }
}
