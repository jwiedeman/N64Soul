use crate::display;
use crate::manifest;
use crate::memory_manager::MemoryManager;
use crate::model::names;
use crate::n64_math;
use crate::{platform::pi, weights};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use core::result::Result;

#[derive(Debug)]
pub enum Error {
    MemoryError,
    RomReadError,
    ComputationError,
    MissingLayer(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MemoryError => write!(f, "Memory allocation error"),
            Error::RomReadError => write!(f, "ROM read error"),
            Error::ComputationError => write!(f, "Computation error"),
            Error::MissingLayer(name) => write!(f, "Missing layer: {}", name),
        }
    }
}

const MAX_SEQ_LENGTH: usize = 128; // Reduced max sequence length

pub struct ModelState<'a> {
    current_layer_weights: Vec<f32>,
    hidden_states: Vec<f32>,
    memory_manager: &'a mut MemoryManager,
    last_checkpoint: Option<usize>,
    manifest: &'a manifest::Manifest,
    dims: crate::model::dims::ModelDims,
    plan: LayerPlan,
}

impl<'a> ModelState<'a> {
    pub fn new(memory_manager: &'a mut MemoryManager, manifest: &'a manifest::Manifest) -> Self {
        let dims = manifest.dims;
        let hidden_states = Vec::with_capacity(MAX_SEQ_LENGTH * dims.d_model as usize);
        let plan = LayerPlan::from_manifest(manifest);
        ModelState {
            current_layer_weights: Vec::new(),
            hidden_states,
            memory_manager,
            last_checkpoint: None,
            manifest,
            dims,
            plan,
        }
    }

    pub fn load_layer_weights(&mut self, layer_idx: usize) -> Result<(), Error> {
        if layer_idx >= self.manifest.layers.len() {
            return Err(Error::MemoryError);
        }

        // Create a checkpoint so the weights can be freed after use
        let cp = self.memory_manager.checkpoint();
        self.last_checkpoint = Some(cp);

        let layer = &self.manifest.layers[layer_idx];
        let offset = layer.offset;
        let size = layer.size as usize;

        self.current_layer_weights = Vec::with_capacity(size / 4); // 4 bytes per f32

        self.read_from_rom(offset, size)?;

        Ok(())
    }

    fn unload_layer_weights(&mut self) {
        if let Some(_cp) = self.last_checkpoint {
            self.memory_manager.pop_checkpoint();
            self.last_checkpoint = None;
        }
        self.current_layer_weights.clear();
    }

    /// Read `size` bytes from ROM at (weights base + offset) using DMA and
    /// convert the bytes into f32 values.
    fn read_from_rom(&mut self, offset: u32, size: usize) -> Result<(), Error> {
        // Allocate temporary buffer for DMA read.
        let buffer_ptr = match self.memory_manager.alloc(size, 4) {
            Some(ptr) => ptr.as_ptr(),
            None => return Err(Error::MemoryError),
        };

        // Perform DMA read via platform PI layer
        let cart_off = weights::weights_rel_to_cart_off(offset as u64);
        let buf = unsafe { core::slice::from_raw_parts_mut(buffer_ptr, size) };
        pi::pi_dma_read(cart_off, buf).map_err(|_| Error::RomReadError)?;
        // Now convert the DMA buffer into f32 values.
        let data = unsafe { core::slice::from_raw_parts(buffer_ptr, size) };
        for chunk in data.chunks(4) {
            if chunk.len() == 4 {
                let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                self.current_layer_weights.push(value);
            }
        }

        Ok(())
    }

    pub fn run_inference(&mut self, input_tokens: &[u32]) -> Result<Vec<u32>, Error> {
        let total_steps = self.plan.layers.len() + 1;
        display::show_progress(0, total_steps);

        let embedding_idx = self
            .plan
            .embedding
            .ok_or(Error::MissingLayer(names::L_TOK_EMB))?;

        self.load_layer_weights(embedding_idx)?;
        self.memory_manager.log_usage("embed_load");
        self.apply_embeddings(input_tokens)?;
        self.memory_manager.log_usage("embed_apply");
        self.unload_layer_weights();
        self.memory_manager.log_usage("embed_unload");

        for (idx, pair) in self.plan.layers.iter().enumerate() {
            self.load_layer_weights(pair.attn_idx)?;
            self.memory_manager.log_usage("attn_load");
            self.apply_attention()?;
            self.memory_manager.log_usage("attn_apply");
            self.unload_layer_weights();
            self.memory_manager.log_usage("attn_unload");

            self.load_layer_weights(pair.ffn_idx)?;
            self.memory_manager.log_usage("ffn_load");
            self.apply_ffn()?;
            self.memory_manager.log_usage("ffn_apply");
            self.unload_layer_weights();
            self.memory_manager.log_usage("ffn_unload");

            display::show_progress(idx + 1, total_steps);
        }

        let output_idx = self
            .plan
            .output
            .ok_or(Error::MissingLayer(names::L_LM_HEAD))?;

        self.load_layer_weights(output_idx)?;
        self.memory_manager.log_usage("out_load");
        let output_tokens = self.generate_output()?;
        self.unload_layer_weights();
        self.memory_manager.log_usage("out_unload");
        display::show_progress(total_steps, total_steps);
        Ok(output_tokens)
    }

    fn apply_embeddings(&mut self, input_tokens: &[u32]) -> Result<(), Error> {
        self.hidden_states.clear();
        let hidden_size = self.dims.d_model as usize;
        let vocab_size = self.dims.vocab_size as usize;
        for &token in input_tokens {
            if token as usize >= vocab_size {
                return Err(Error::ComputationError);
            }

            let embed_offset = token as usize * hidden_size;
            if embed_offset + hidden_size > self.current_layer_weights.len() {
                return Err(Error::MemoryError);
            }

            for i in 0..hidden_size {
                self.hidden_states
                    .push(self.current_layer_weights[embed_offset + i]);
            }
        }

        Ok(())
    }

    fn apply_attention(&mut self) -> Result<(), Error> {
        let hidden_size = self.dims.d_model as usize;
        let seq_len = self.hidden_states.len() / hidden_size;
        let mat_size = hidden_size * hidden_size;

        // Ensure we have enough weights for a single-head attention layer
        if self.current_layer_weights.len() < mat_size * 4 {
            return Err(Error::MemoryError);
        }

        let (q_w, rest) = self.current_layer_weights.split_at(mat_size);
        let (k_w, rest) = rest.split_at(mat_size);
        let (v_w, o_w) = rest.split_at(mat_size);

        let mut q = vec![0.0f32; seq_len * hidden_size];
        let mut k = vec![0.0f32; seq_len * hidden_size];
        let mut v = vec![0.0f32; seq_len * hidden_size];

        // Linear projections for Q, K, V
        for t in 0..seq_len {
            for h in 0..hidden_size {
                let mut q_sum = 0.0;
                let mut k_sum = 0.0;
                let mut v_sum = 0.0;
                for i in 0..hidden_size {
                    let x = self.hidden_states[t * hidden_size + i];
                    q_sum += x * q_w[h * hidden_size + i];
                    k_sum += x * k_w[h * hidden_size + i];
                    v_sum += x * v_w[h * hidden_size + i];
                }
                q[t * hidden_size + h] = q_sum;
                k[t * hidden_size + h] = k_sum;
                v[t * hidden_size + h] = v_sum;
            }
        }

        let mut attended = vec![0.0f32; seq_len * hidden_size];
        let scale = n64_math::sqrt(hidden_size as f32);

        for t in 0..seq_len {
            let mut scores = vec![0.0f32; seq_len];
            let mut sum = 0.0f32;
            for s in 0..seq_len {
                let mut dot = 0.0;
                for h in 0..hidden_size {
                    dot += q[t * hidden_size + h] * k[s * hidden_size + h];
                }
                let score = n64_math::exp_approx(dot / scale);
                scores[s] = score;
                sum += score;
            }

            for s in 0..seq_len {
                let weight = scores[s] / sum;
                for h in 0..hidden_size {
                    attended[t * hidden_size + h] += weight * v[s * hidden_size + h];
                }
            }
        }

        let mut new_states = vec![0.0f32; seq_len * hidden_size];
        for t in 0..seq_len {
            for h in 0..hidden_size {
                let mut sum = 0.0;
                for i in 0..hidden_size {
                    sum += attended[t * hidden_size + i] * o_w[h * hidden_size + i];
                }
                new_states[t * hidden_size + h] = sum;
            }
        }

        self.hidden_states = new_states;
        Ok(())
    }

    fn apply_ffn(&mut self) -> Result<(), Error> {
        let hidden_size = self.dims.d_model as usize;
        let seq_len = self.hidden_states.len() / hidden_size;
        let mat_size = hidden_size * hidden_size;

        if self.current_layer_weights.len() < mat_size * 2 {
            return Err(Error::MemoryError);
        }

        let (w1, w2) = self.current_layer_weights.split_at(mat_size);

        let mut hidden = vec![0.0f32; seq_len * hidden_size];
        for t in 0..seq_len {
            for h in 0..hidden_size {
                let mut sum = 0.0;
                for i in 0..hidden_size {
                    sum += self.hidden_states[t * hidden_size + i] * w1[h * hidden_size + i];
                }
                hidden[t * hidden_size + h] = if sum > 0.0 { sum } else { 0.0 };
            }
        }

        let mut output = vec![0.0f32; seq_len * hidden_size];
        for t in 0..seq_len {
            for h in 0..hidden_size {
                let mut sum = 0.0;
                for i in 0..hidden_size {
                    sum += hidden[t * hidden_size + i] * w2[h * hidden_size + i];
                }
                output[t * hidden_size + h] = sum;
            }
        }

        self.hidden_states = output;
        Ok(())
    }

    fn generate_output(&self) -> Result<Vec<u32>, Error> {
        let hidden_size = self.dims.d_model as usize;
        let vocab_size = self.dims.vocab_size as usize;
        let seq_len = self.hidden_states.len() / hidden_size;
        if seq_len == 0 {
            return Err(Error::ComputationError);
        }

        let last_pos = (seq_len - 1) * hidden_size;
        let mut logits = Vec::with_capacity(vocab_size);

        for v in 0..vocab_size {
            let mut logit = 0.0;
            for h in 0..hidden_size {
                if h < self.hidden_states.len() - last_pos {
                    let weight_idx = v * hidden_size + h;
                    if weight_idx < self.current_layer_weights.len() {
                        logit += self.hidden_states[last_pos + h]
                            * self.current_layer_weights[weight_idx];
                    }
                }
            }
            logits.push(logit);
        }

        let mut max_idx = 0;
        let mut max_val = f32::MIN;
        for (i, &logit) in logits.iter().enumerate() {
            if logit > max_val {
                max_val = logit;
                max_idx = i;
            }
        }

        Ok(vec![max_idx as u32])
    }
}

struct LayerPair {
    attn_idx: usize,
    ffn_idx: usize,
}

struct LayerPlan {
    embedding: Option<usize>,
    output: Option<usize>,
    layers: Vec<LayerPair>,
}

impl LayerPlan {
    fn from_manifest(manifest: &manifest::Manifest) -> Self {
        let mut embedding = None;
        let mut output = None;
        let mut attn: Vec<(usize, usize)> = Vec::new();
        let mut ffn: Vec<(usize, usize)> = Vec::new();
        let mut attn_fallback = 0usize;
        let mut ffn_fallback = 0usize;

        for (idx, layer) in manifest.layers.iter().enumerate() {
            let name = layer.name.as_str();
            if embedding.is_none() && is_embedding_layer(name) {
                embedding = Some(idx);
                continue;
            }
            if output.is_none() && is_output_layer(name) {
                output = Some(idx);
                continue;
            }
            if is_attention_layer(name) {
                let order = parse_layer_index(name).unwrap_or_else(|| {
                    let v = fallback_order(attn_fallback);
                    attn_fallback += 1;
                    v
                });
                attn.push((order, idx));
                continue;
            }
            if is_ffn_layer(name) {
                let order = parse_layer_index(name).unwrap_or_else(|| {
                    let v = fallback_order(ffn_fallback);
                    ffn_fallback += 1;
                    v
                });
                ffn.push((order, idx));
            }
        }

        attn.sort_by_key(|(order, _)| *order);
        ffn.sort_by_key(|(order, _)| *order);
        let mut layers = Vec::new();
        for (a, f) in attn.into_iter().zip(ffn.into_iter()) {
            layers.push(LayerPair {
                attn_idx: a.1,
                ffn_idx: f.1,
            });
        }

        LayerPlan {
            embedding,
            output,
            layers,
        }
    }
}

fn fallback_order(fallback: usize) -> usize {
    0x1000 + fallback
}

fn is_embedding_layer(name: &str) -> bool {
    name == names::L_TOK_EMB || name.contains("tok_emb")
}

fn is_output_layer(name: &str) -> bool {
    name == names::L_LM_HEAD || name.ends_with("lm_head")
}

fn is_attention_layer(name: &str) -> bool {
    name.contains("attn") || name.contains("attention")
}

fn is_ffn_layer(name: &str) -> bool {
    name.contains("ffn") || name.contains("mlp") || name.contains("feed_forward")
}

fn parse_layer_index(name: &str) -> Option<usize> {
    if let Some(pos) = name.find(".h.") {
        let mut value: usize = 0;
        let mut found = false;
        for ch in name[pos + 3..].chars() {
            if ch.is_ascii_digit() {
                found = true;
                value = value * 10 + (ch as u8 - b'0') as usize;
            } else {
                break;
            }
        }
        if found {
            return Some(value);
        }
    }
    if let Some(pos) = name.find("layer") {
        let mut idx = pos + 5;
        let bytes = name.as_bytes();
        if idx < bytes.len() && (bytes[idx] == b'_' || bytes[idx] == b'.') {
            idx += 1;
        }
        let mut value: usize = 0;
        let mut found = false;
        while idx < bytes.len() {
            let b = bytes[idx];
            if b.is_ascii_digit() {
                found = true;
                value = value * 10 + (b - b'0') as usize;
                idx += 1;
            } else {
                break;
            }
        }
        if found {
            return Some(value);
        }
    }
    None
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn load_layer_out_of_bounds() {
        let mut mm = MemoryManager::new_for_test();
        let manifest = manifest::Manifest {
            layers: vec![],
            align: 64,
            dims: crate::model::dims::ModelDims::new(16, 32),
        };
        let mut state = ModelState::new(&mut mm, &manifest);
        let res = state.load_layer_weights(0);
        assert!(matches!(res, Err(Error::MemoryError)));
    }

    #[test]
    fn apply_embeddings_invalid_token() {
        let mut mm = MemoryManager::new_for_test();
        let manifest = manifest::Manifest {
            layers: vec![manifest::Layer {
                name: names::L_TOK_EMB.to_string(),
                offset: 0,
                size: 64,
            }],
            align: 64,
            dims: crate::model::dims::ModelDims::new(16, 32),
        };
        let mut state = ModelState::new(&mut mm, &manifest);
        state.current_layer_weights = vec![0.0; 16];
        let res = state.apply_embeddings(&[manifest.dims.vocab_size]);
        assert!(matches!(res, Err(Error::ComputationError)));
    }

    #[test]
    fn generate_output_empty_state() {
        let mut mm = MemoryManager::new_for_test();
        let manifest = manifest::Manifest {
            layers: vec![],
            align: 64,
            dims: crate::model::dims::ModelDims::new(16, 32),
        };
        let state = ModelState::new(&mut mm, &manifest);
        let res = state.generate_output();
        assert!(matches!(res, Err(Error::ComputationError)));
    }
}
