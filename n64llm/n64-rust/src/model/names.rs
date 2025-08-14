pub const L_TOK_EMB: &str = "tok_embeddings";   // row-major [vocab, d_model], f32 LE
pub const L_LM_HEAD: &str = "lm_head";          // row-major [vocab, d_model], f32 LE (no bias)
pub const L_MODEL_META: &str = "model_meta";    // optional: small binary meta
