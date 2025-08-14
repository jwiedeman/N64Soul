#[derive(Debug, Copy, Clone)]
pub struct ModelDims { pub d_model: u32, pub vocab_size: u32 }

impl ModelDims {
    pub const fn new(d_model: u32, vocab_size: u32) -> Self { Self { d_model, vocab_size } }
}
