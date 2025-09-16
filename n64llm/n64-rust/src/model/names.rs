pub const L_TOK_EMB: &str = "tok_embeddings"; // row-major [vocab, d_model], f32 LE
pub const L_POS_EMB: &str = "pos_embeddings"; // row-major [n_positions, d_model], f32 LE
pub const L_LM_HEAD: &str = "lm_head"; // row-major [vocab, d_model], f32 LE (no bias)
pub const L_FINAL_NORM_WEIGHT: &str = "ln_f.weight"; // [d_model]
pub const L_FINAL_NORM_BIAS: &str = "ln_f.bias"; // [d_model]
pub const L_TOKENIZER_MODEL: &str = "tokenizer.model"; // packed tokenizer assets
pub const L_MODEL_META: &str = "model_meta"; // optional: small binary meta
