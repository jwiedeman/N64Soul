use crate::{model::dims::ModelDims, weights_manifest::ManifestView};
use crate::weights_manifest_find::find;
use crate::weights::weights_rel_to_cart_off;
use crate::io::rom_reader::RomReader;

pub fn load_dims_from_meta<R: RomReader>(rr: &mut R, man: &ManifestView<'_>) -> Option<ModelDims> {
    let e = find(man, crate::model::names::L_MODEL_META)?;
    if e.size < 12 { return None; }
    let mut buf = [0u8; 12];
    let ok = rr.read(weights_rel_to_cart_off(e.offset as u64), &mut buf);
    if !ok { return None; }
    let magic = u32::from_le_bytes(buf[0..4].try_into().unwrap());
    if magic != 0x4D455441 { return None; } // 'META'
    let d_model = u32::from_le_bytes(buf[4..8].try_into().unwrap());
    let vocab   = u32::from_le_bytes(buf[8..12].try_into().unwrap());
    Some(ModelDims::new(d_model, vocab))
}
