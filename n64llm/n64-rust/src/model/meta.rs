use crate::io::rom_reader::RomReader;
use crate::model::dims::ModelDims;
use crate::weights::weights_rel_to_cart_off;
use crate::weights_manifest::ManifestView;
use crate::weights_manifest_find::find;

const META_MAGIC: u32 = 0x4D45_5441; // 'META'
const META_VERSION_V1: u32 = 1;
const META_BYTES_V1: usize = 32;

pub fn load_dims_from_meta<R: RomReader>(rr: &mut R, man: &ManifestView<'_>) -> Option<ModelDims> {
    let e = find(man, crate::model::names::L_MODEL_META)?;
    if e.size < META_BYTES_V1 as u32 {
        return None;
    }
    let mut buf = [0u8; META_BYTES_V1];
    if !rr.read(weights_rel_to_cart_off(e.offset as u64), &mut buf) {
        return None;
    }
    let magic = u32::from_le_bytes(buf[0..4].try_into().unwrap());
    if magic != META_MAGIC {
        return None;
    }
    let version = u32::from_le_bytes(buf[4..8].try_into().unwrap());
    if version != META_VERSION_V1 {
        return None;
    }
    let d_model = u32::from_le_bytes(buf[8..12].try_into().unwrap());
    let vocab = u32::from_le_bytes(buf[12..16].try_into().unwrap());
    let n_layer = u32::from_le_bytes(buf[16..20].try_into().unwrap());
    let n_head = u32::from_le_bytes(buf[20..24].try_into().unwrap());
    let n_positions = u32::from_le_bytes(buf[24..28].try_into().unwrap());
    let d_ff = u32::from_le_bytes(buf[28..32].try_into().unwrap());
    Some(ModelDims::new(
        d_model,
        vocab,
        n_layer,
        n_head,
        n_positions,
        d_ff,
    ))
}
