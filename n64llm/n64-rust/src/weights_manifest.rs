#![allow(dead_code)]

#[link_section = ".model_manifest"]
#[used]
pub static MODEL_MANIFEST: [u8; { include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/assets/weights.manifest.bin")).len() }] =
    *include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/weights.manifest.bin"));

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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn v1_and_v2_manifest_parse() {
        // v1: magic, ver=1, align=64, count=1, entry("a", off=64, sz=3)
        let mut v1: Vec<u8> = b"N64W".to_vec();
        v1.extend(&1u16.to_le_bytes());
        v1.extend(&64u16.to_le_bytes());
        v1.extend(&1u32.to_le_bytes());
        v1.extend(&(1u16.to_le_bytes()));
        v1.push(b'a');
        v1.extend(&64u32.to_le_bytes());
        v1.extend(&3u32.to_le_bytes());
        let m1 = ManifestView::new(&v1).unwrap();
        assert_eq!(m1.version(), 1);
        let mut seen = false;
        m1.for_each(|e| {
            seen = true;
            assert_eq!(e.name, "a");
            assert_eq!(e.offset, 64);
            assert_eq!(e.size, 3);
            assert!(e.crc32.is_none());
            true
        }).unwrap();
        assert!(seen);

        // v2 adds CRC
        let mut v2 = v1.clone();
        v2[4] = 2; // set version=2
        v2.extend(&0xDEADBEEFu32.to_le_bytes());
        let m2 = ManifestView::new(&v2).unwrap();
        assert_eq!(m2.version(), 2);
        let mut crc_ok = false;
        m2.for_each(|e| {
            crc_ok = e.crc32 == Some(0xDEADBEEF);
            true
        }).unwrap();
        assert!(crc_ok);
    }
}
