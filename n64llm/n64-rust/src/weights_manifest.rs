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
}

pub struct ManifestView<'a> {
    bytes: &'a [u8],
    align: u16,
    count: u32,
    off_entries: usize,
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
        if ver != 1 { return Err(ManErr::BadVersion); }
        let align = rd_u16_le(bytes, &mut i)?;
        let count = rd_u32_le(bytes, &mut i)?;
        Ok(Self { bytes, align, count, off_entries: i })
    }
    pub fn align(&self) -> u16 { self.align }
    pub fn count(&self) -> u32 { self.count }

    pub fn for_each<F: FnMut(Entry<'a>) -> bool>(&self, mut f: F) -> Result<(), ManErr> {
        let mut i = self.off_entries;
        for _ in 0..self.count {
            let nlen = rd_u16_le(self.bytes, &mut i)? as usize;
            if i + nlen + 8 > self.bytes.len() { return Err(ManErr::Truncated); }
            let name_bytes = &self.bytes[i..i+nlen]; i += nlen;
            let off = u32::from_le_bytes(self.bytes[i..i+4].try_into().unwrap()); i += 4;
            let sz  = u32::from_le_bytes(self.bytes[i..i+4].try_into().unwrap()); i += 4;
            let name = core::str::from_utf8(name_bytes).map_err(|_| ManErr::Utf8)?;
            if !f(Entry { name, offset: off, size: sz }) { break; }
        }
        Ok(())
    }
}
