use crate::infer::{decoder::argmax_over_head, embedding::gather_embedding};
use crate::model::dims::ModelDims;
use crate::model::meta::load_dims_from_meta;
use crate::weights_manifest::ManifestView;
use crate::{display, io::rom_reader::RomReader};
use alloc::format;
use alloc::vec;

pub fn run<R: RomReader>(rr: &mut R, man_bytes: &'static [u8], seed_token: u32) {
    display::print_line("=== DECODE ONCE ===");

    let man = match ManifestView::new(man_bytes) {
        Ok(v) => v,
        Err(_) => {
            display::print_line("Manifest parse ERR");
            return;
        }
    };

    let dims = load_dims_from_meta(rr, &man).unwrap_or_else(ModelDims::fallback);
    display::print_line(&format!(
        "dims: d_model={} vocab={}",
        dims.d_model, dims.vocab_size
    ));

    let mut h = vec![0.0f32; dims.d_model as usize];
    let mut row = vec![0u8; (dims.d_model as usize) * 4];

    if !gather_embedding(rr, &man, &dims, seed_token, &mut h) {
        display::print_line("Embedding: FAIL");
        return;
    }

    match argmax_over_head(rr, &man, &dims, &h, &mut row) {
        None => display::print_line("Decoder: FAIL"),
        Some(st) => {
            display::print_line(&format!(
                "next_token={}  logit={:.3}  scanned={}",
                st.best_id, st.best_logit, st.scanned
            ));
        }
    }
}
