use crate::{model::dims::ModelDims, weights_manifest::ManifestView};
use crate::weights_manifest_find::find;

pub fn infer_from_layers(man: &ManifestView<'_>) -> Option<ModelDims> {
    let _e_emb  = find(man, crate::model::names::L_TOK_EMB)?;
    let _e_head = find(man, crate::model::names::L_LM_HEAD)?;
    // Both are vocab*d_model*4 (f32 LE). We can’t get vocab and d_model uniquely from sizes alone.
    // For now, assume they’re equal and we rely on a known vocab, or we provide META soon.
    None
}
