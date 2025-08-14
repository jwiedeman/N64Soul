use crate::{io::rom_reader::RomReader, weights_manifest::ManifestView};
use crate::weights_manifest_find::find;
use crate::weights::weights_rel_to_cart_off;
use crate::util::f32le::load_f32le_slice;
use crate::model::dims::ModelDims;
use alloc::vec;

pub fn gather_embedding<R: RomReader>(
    rr: &mut R,
    man: &ManifestView<'_>,
    dims: &ModelDims,
    token_id: u32,
    out_h: &mut [f32],              // len = d_model
) -> bool {
    let e = match find(man, crate::model::names::L_TOK_EMB) { Some(v) => v, None => return false };
    let d_model = dims.d_model as usize;
    if out_h.len() < d_model { return false; }

    // row offset in bytes: token_id * d_model * 4
    let row_bytes = (d_model * 4) as u64;
    let off_rel   = (token_id as u64) * row_bytes;
    if off_rel + row_bytes > e.size as u64 { return false; }

    let mut buf = vec![0u8; d_model * 4];
    let cart_off = weights_rel_to_cart_off(e.offset as u64 + off_rel);
    if !rr.read(cart_off, &mut buf) { return false; }

    load_f32le_slice(out_h, &buf);
    true
}
