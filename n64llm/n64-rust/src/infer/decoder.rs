use crate::{io::rom_reader::RomReader, weights_manifest::ManifestView};
use crate::weights_manifest_find::find;
use crate::weights::weights_rel_to_cart_off;
use crate::config::BURST_BYTES;
use crate::util::f32le::dot_f32le_row;
use crate::model::dims::ModelDims;

pub struct DecodeStats {
    pub best_id: u32,
    pub best_logit: f32,
    pub scanned: u32,
}

pub fn argmax_over_head<R: RomReader>(
    rr: &mut R,
    man: &ManifestView<'_>,
    dims: &ModelDims,
    h: &[f32],                // len = d_model
    scratch_row: &mut [u8],   // len >= d_model*4
) -> Option<DecodeStats> {
    let e = find(man, crate::model::names::L_LM_HEAD)?;
    let d_model = dims.d_model as usize;
    let vocab   = dims.vocab_size as usize;

    if h.len() < d_model || scratch_row.len() < d_model*4 { return None; }

    let row_bytes = (d_model * 4) as u64;
    if (row_bytes * vocab as u64) > e.size as u64 { return None; }

    let mut best_id: u32 = 0;
    let mut best_logit = f32::NEG_INFINITY;
    let mut scanned: u32 = 0;

    for i in 0..vocab {
        let off_rel = (i as u64) * row_bytes;
        let cart_off = weights_rel_to_cart_off(e.offset as u64 + off_rel);

        let to_read = d_model * 4;
        if !rr.read(cart_off, &mut scratch_row[..to_read]) { return None; }

        let logit = dot_f32le_row(&scratch_row[..to_read], h);
        if logit > best_logit {
            best_logit = logit;
            best_id = i as u32;
        }
        scanned += 1;

        if (i % (BURST_BYTES / 64).max(1)) == 0 {
        }
    }
    Some(DecodeStats { best_id, best_logit, scanned })
}
