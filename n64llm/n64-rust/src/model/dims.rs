#[derive(Debug, Copy, Clone)]
pub struct ModelDims {
    pub d_model: u32,
    pub vocab_size: u32,
    pub n_layer: u32,
    pub n_head: u32,
    pub n_positions: u32,
    pub d_ff: u32,
}

impl ModelDims {
    pub const fn new(
        d_model: u32,
        vocab_size: u32,
        n_layer: u32,
        n_head: u32,
        n_positions: u32,
        d_ff: u32,
    ) -> Self {
        Self {
            d_model,
            vocab_size,
            n_layer,
            n_head,
            n_positions,
            d_ff,
        }
    }

    pub const fn fallback() -> Self {
        Self::new(
            crate::model::config::D_MODEL_FALLBACK,
            crate::model::config::VOCAB_FALLBACK,
            crate::model::config::N_LAYER_FALLBACK,
            crate::model::config::N_HEAD_FALLBACK,
            crate::model::config::N_POS_FALLBACK,
            crate::model::config::D_FF_FALLBACK,
        )
    }
}
