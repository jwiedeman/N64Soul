#![allow(dead_code)]

#[cfg(feature = "embed_assets")]
#[link_section = ".model_manifest"]
#[used]
pub static MODEL_MANIFEST: [u8; { include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/assets/weights.manifest.bin")).len() }] =
    *include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/weights.manifest.bin"));

#[cfg(any(test, feature = "host"))]
extern crate alloc;
#[cfg(any(test, feature = "host"))]
use alloc::vec::Vec;
#[cfg(any(test, feature = "host"))]
use alloc::boxed::Box;

#[cfg(any(test, feature = "host"))]
use core::sync::atomic::{AtomicPtr, Ordering};

#[cfg(any(test, feature = "host"))]
fn man_bytes_host() -> &'static [u8] {
    // Tiny v2 manifest with 2 entries; used only in unit tests if assets are absent.
    // (Constructed once; static for lifetime)
    static P: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
    unsafe {
        if P.load(Ordering::Relaxed).is_null() {
            let mut v = Vec::<u8>::new();
            v.extend_from_slice(b"N64W");
            v.extend_from_slice(&2u16.to_le_bytes());        // ver=2
            v.extend_from_slice(&64u16.to_le_bytes());       // align
            v.extend_from_slice(&2u32.to_le_bytes());        // count
            let push = |v: &mut Vec<u8>, name: &str, off: u32, sz: u32, crc: u32| {
                let nb = name.as_bytes();
                v.extend_from_slice(&(nb.len() as u16).to_le_bytes());
                v.extend_from_slice(nb);
                v.extend_from_slice(&off.to_le_bytes());
                v.extend_from_slice(&sz.to_le_bytes());
                v.extend_from_slice(&crc.to_le_bytes());
            };
            push(&mut v, "tok", 64, 16, 0x4D6F28D3);
            push(&mut v, "ffn", 128, 4, 0xD202EF8D);
            let b = v.into_boxed_slice();
            let p = Box::into_raw(b) as *mut u8;
            P.store(p, Ordering::Relaxed);
        }
        let p = P.load(Ordering::Relaxed);
        core::slice::from_raw_parts(p, 4 + 2 + 2 + 4 + (2 + 3 + 4 + 4 + 4)*2)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Entry<'a> {
    pub name: &'a str,
    pub offset: u32,
    pub size: u32,
    pub crc32: Option<u32>,
}

pub struct ManifestView<'a> {
    bytes: &'a [u8],
    align: u16,
    count: u32,
    off_entries: usize,
    ver: u16,
}

#[derive(Debug)]
pub enum ManErr { BadMagic, BadVersion, Truncated, Utf8 }

fn rd_u16_le(b: &[u8], i: &mut usize) -> Result<u16, ManErr> {
    if *i + 2 > b.len() { return Err(ManErr::Truncated); }
    let v = u16::from_le_bytes([b[*i], b[*i+1]]); *i += 2; Ok(v)
}
fn rd_u32_le(b: &[u8], i: &mut usize) -> Result<u32, ManErr> {
    if *i + 4 > b.len() { return Err(ManErr::Truncated); }
    let v = u32::from_le_bytes([b[*i], b[*i+1], b[*i+2], b[*i+3]]); *i += 4; Ok(v)
}

impl<'a> ManifestView<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, ManErr> {
        let mut i = 0;
        if bytes.len() < 12 { return Err(ManErr::Truncated); }
        if &bytes[0..4] != b"N64W" { return Err(ManErr::BadMagic); }
        i = 4;
        let ver = rd_u16_le(bytes, &mut i)?;
        if ver != 1 && ver != 2 { return Err(ManErr::BadVersion); }
        let align = rd_u16_le(bytes, &mut i)?;
        let count = rd_u32_le(bytes, &mut i)?;
        Ok(Self { bytes, align, count, off_entries: i, ver })
    }
    pub fn align(&self) -> u16 { self.align }
    pub fn count(&self) -> u32 { self.count }
    pub fn version(&self) -> u16 { self.ver }

    pub fn for_each<F: FnMut(Entry<'a>) -> bool>(&self, mut f: F) -> Result<(), ManErr> {
        let mut i = self.off_entries;
        for _ in 0..self.count {
            let nlen = rd_u16_le(self.bytes, &mut i)? as usize;
            if i + nlen + 8 > self.bytes.len() { return Err(ManErr::Truncated); }
            let name_bytes = &self.bytes[i..i+nlen]; i += nlen;
            let off = u32::from_le_bytes(self.bytes[i..i+4].try_into().unwrap()); i += 4;
            let sz  = u32::from_le_bytes(self.bytes[i..i+4].try_into().unwrap()); i += 4;
            let crc32 = if self.ver >= 2 {
                if i + 4 > self.bytes.len() { return Err(ManErr::Truncated); }
                let c = u32::from_le_bytes(self.bytes[i..i+4].try_into().unwrap()); i += 4; Some(c)
            } else {
                None
            };
            let name = core::str::from_utf8(name_bytes).map_err(|_| ManErr::Utf8)?;
            if !f(Entry { name, offset: off, size: sz, crc32 }) { break; }
        }
        Ok(())
    }
}

pub fn manifest() -> Option<ManifestView<'static>> {
    #[cfg(feature = "embed_assets")]
    {
        ManifestView::new(&MODEL_MANIFEST).ok()
    }
    #[cfg(not(feature = "embed_assets"))]
    {
        ManifestView::new(man_bytes_host()).ok()
    }
}

#[cfg(test)]
mod t_manifest {
    use super::*;

    #[test]
    fn parses_v1_and_v2() {
        // v1 blob
        let mut v1 = b"N64W".to_vec();
        v1.extend(&1u16.to_le_bytes()); v1.extend(&64u16.to_le_bytes());
        v1.extend(&1u32.to_le_bytes());
        v1.extend(&1u16.to_le_bytes()); v1.extend(b"a");
        v1.extend(&64u32.to_le_bytes()); v1.extend(&3u32.to_le_bytes());
        let m1 = ManifestView::new(&v1).unwrap();
        assert_eq!(m1.version(), 1);

        // v2 blob = v1 + crc
        let mut v2 = v1.clone(); v2[4] = 2; // version=2
        v2.extend(&0xDEADBEEFu32.to_le_bytes());
        let m2 = ManifestView::new(&v2).unwrap();
        assert_eq!(m2.version(), 2);
    }
}
