use alloc::vec::Vec;
use core::result::Result;
use core::fmt;
use crate::memory_manager::MemoryManager;
use crate::display;
use crate::n64_sys::{PI_STATUS_REG, PI_STATUS_IO_BUSY, PI_STATUS_DMA_BUSY, pi_read};
use alloc::vec;
use crate::n64_math;

#[derive(Debug)]
pub enum Error {
    MemoryError,
    RomReadError,
    ComputationError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MemoryError => write!(f, "Memory allocation error"),
            Error::RomReadError => write!(f, "ROM read error"),
            Error::ComputationError => write!(f, "Computation error"),
        }
    }
}

// Constants for model configuration
const NUM_LAYERS: usize = 6;  // Reduced from standard DistilGPT2
const HIDDEN_SIZE: usize = 384; // Reduced for memory constraints
const VOCAB_SIZE: usize = 25000; // Approximate for GPT-2 
const MAX_SEQ_LENGTH: usize = 128; // Reduced max sequence length

// Offsets and sizes (in bytes) of each weight segment in ROM.
// These must match the layout produced by your export script.
const LAYER_OFFSETS: [u32; 14] = [
    0x00000000, // Embedding layer
    0x00100000, // Layer 0 Attention
    0x00200000, // Layer 0 FFN
    0x00300000, // Layer 1 Attention
    0x00400000, // Layer 1 FFN
    0x00500000, // Layer 2 Attention
    0x00600000, // Layer 2 FFN
    0x00700000, // Layer 3 Attention
    0x00800000, // Layer 3 FFN
    0x00900000, // Layer 4 Attention
    0x00A00000, // Layer 4 FFN
    0x00B00000, // Layer 5 Attention
    0x00C00000, // Layer 5 FFN
    0x00D00000, // Output layer
];

// Layer sizes in bytes (approximate, should match export script)
const LAYER_SIZES: [usize; 14] = [
    1024 * 1024,  // 1MB for embedding
    1024 * 1024,  // 1MB per attention layer
    1024 * 1024,  // 1MB per FFN
    1024 * 1024,
    1024 * 1024,
    1024 * 1024,
    1024 * 1024,
    1024 * 1024,
    1024 * 1024,
    1024 * 1024,
    1024 * 1024,
    1024 * 1024,
    1024 * 1024,
    1024 * 1024,  // 1MB for output layer
];

pub struct ModelState<'a> {
    current_layer_weights: Vec<f32>,
    layer_index: usize,
    hidden_states: Vec<f32>,
    memory_manager: &'a mut MemoryManager,
    last_checkpoint: Option<usize>,
}

impl<'a> ModelState<'a> {
    pub fn new(memory_manager: &'a mut MemoryManager) -> Self {
        let hidden_states = Vec::with_capacity(MAX_SEQ_LENGTH * HIDDEN_SIZE);
        ModelState {
            current_layer_weights: Vec::new(),
            layer_index: 0,
            hidden_states,
            memory_manager,
            last_checkpoint: None,
        }
    }
    
    pub fn load_layer_weights(&mut self, layer_idx: usize) -> Result<(), Error> {
        if layer_idx >= LAYER_OFFSETS.len() {
            return Err(Error::MemoryError);
        }

        // Create a checkpoint so the weights can be freed after use
        let cp = self.memory_manager.checkpoint();
        self.last_checkpoint = Some(cp);

        let offset = LAYER_OFFSETS[layer_idx];
        let size = LAYER_SIZES[layer_idx];

        self.current_layer_weights = Vec::with_capacity(size / 4); // 4 bytes per f32
        
        self.read_from_rom(offset, size)?;
        
        self.layer_index = layer_idx;
        Ok(())
    }

    fn unload_layer_weights(&mut self) {
        if let Some(_cp) = self.last_checkpoint {
            self.memory_manager.pop_checkpoint();
            self.last_checkpoint = None;
        }
        self.current_layer_weights.clear();
    }
    
    /// Read `size` bytes from ROM at (base address + offset) using DMA.
    /// We allocate a temporary buffer in RDRAM, call pi_read to DMA data from ROM into it,
    /// then convert the bytes into f32 values.
    fn read_from_rom(&mut self, offset: u32, size: usize) -> Result<(), Error> {
        // Allocate temporary buffer for DMA read.
        let buffer_ptr = match self.memory_manager.alloc(size, 4) {
            Some(ptr) => ptr.as_ptr(),
            None => return Err(Error::MemoryError),
        };

        // Calculate the ROM address: weights are expected at base 0x10000000 + offset.
        let rom_address = 0x10000000u32 + offset;
        
        // Wait until any previous DMA is complete (optional if pi_read does that)
        let pi_status_reg = unsafe { &*(PI_STATUS_REG as *const u32) };
        while (*pi_status_reg & (PI_STATUS_IO_BUSY | PI_STATUS_DMA_BUSY)) != 0 {
            // Busy wait
        }
        
        // Perform DMA read: read `size` bytes from ROM into buffer_ptr.
        unsafe {
            pi_read(buffer_ptr, rom_address, size as u32);
        }
        
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
        display::show_progress(0, NUM_LAYERS + 1);
        self.load_layer_weights(0)?;
        self.memory_manager.log_usage("embed_load");
        self.apply_embeddings(input_tokens)?;
        self.memory_manager.log_usage("embed_apply");
        self.unload_layer_weights();
        self.memory_manager.log_usage("embed_unload");

        for layer_idx in 0..NUM_LAYERS {
            self.load_layer_weights(layer_idx * 2 + 1)?;
            self.memory_manager.log_usage("attn_load");
            self.apply_attention()?;
            self.memory_manager.log_usage("attn_apply");
            self.unload_layer_weights();
            self.memory_manager.log_usage("attn_unload");

            self.load_layer_weights(layer_idx * 2 + 2)?;
            self.memory_manager.log_usage("ffn_load");
            self.apply_ffn()?;
            self.memory_manager.log_usage("ffn_apply");
            self.unload_layer_weights();
            self.memory_manager.log_usage("ffn_unload");

            display::show_progress(layer_idx + 1, NUM_LAYERS + 1);
        }

        self.load_layer_weights(NUM_LAYERS * 2 + 1)?;
        self.memory_manager.log_usage("out_load");
        let output_tokens = self.generate_output()?;
        self.unload_layer_weights();
        self.memory_manager.log_usage("out_unload");
        display::show_progress(NUM_LAYERS + 1, NUM_LAYERS + 1);
        Ok(output_tokens)
    }
    
    fn apply_embeddings(&mut self, input_tokens: &[u32]) -> Result<(), Error> {
        self.hidden_states.clear();
        
        for &token in input_tokens {
            if token as usize >= VOCAB_SIZE {
                return Err(Error::ComputationError);
            }
            
            let embed_offset = token as usize * HIDDEN_SIZE;
            if embed_offset + HIDDEN_SIZE > self.current_layer_weights.len() {
                return Err(Error::MemoryError);
            }
            
            for i in 0..HIDDEN_SIZE {
                self.hidden_states.push(self.current_layer_weights[embed_offset + i]);
            }
        }
        
        Ok(())
    }
    
    fn apply_attention(&mut self) -> Result<(), Error> {
        let seq_len = self.hidden_states.len() / HIDDEN_SIZE;
        let mat_size = HIDDEN_SIZE * HIDDEN_SIZE;

        // Ensure we have enough weights for a single-head attention layer
        if self.current_layer_weights.len() < mat_size * 4 {
            return Err(Error::MemoryError);
        }

        let (q_w, rest) = self.current_layer_weights.split_at(mat_size);
        let (k_w, rest) = rest.split_at(mat_size);
        let (v_w, o_w) = rest.split_at(mat_size);

        let mut q = vec![0.0f32; seq_len * HIDDEN_SIZE];
        let mut k = vec![0.0f32; seq_len * HIDDEN_SIZE];
        let mut v = vec![0.0f32; seq_len * HIDDEN_SIZE];

        // Linear projections for Q, K, V
        for t in 0..seq_len {
            for h in 0..HIDDEN_SIZE {
                let mut q_sum = 0.0;
                let mut k_sum = 0.0;
                let mut v_sum = 0.0;
                for i in 0..HIDDEN_SIZE {
                    let x = self.hidden_states[t * HIDDEN_SIZE + i];
                    q_sum += x * q_w[h * HIDDEN_SIZE + i];
                    k_sum += x * k_w[h * HIDDEN_SIZE + i];
                    v_sum += x * v_w[h * HIDDEN_SIZE + i];
                }
                q[t * HIDDEN_SIZE + h] = q_sum;
                k[t * HIDDEN_SIZE + h] = k_sum;
                v[t * HIDDEN_SIZE + h] = v_sum;
            }
        }

        let mut attended = vec![0.0f32; seq_len * HIDDEN_SIZE];
        let scale = n64_math::sqrt(HIDDEN_SIZE as f32);

        for t in 0..seq_len {
            let mut scores = vec![0.0f32; seq_len];
            let mut sum = 0.0f32;
            for s in 0..seq_len {
                let mut dot = 0.0;
                for h in 0..HIDDEN_SIZE {
                    dot += q[t * HIDDEN_SIZE + h] * k[s * HIDDEN_SIZE + h];
                }
                let score = n64_math::exp_approx(dot / scale);
                scores[s] = score;
                sum += score;
            }

            for s in 0..seq_len {
                let weight = scores[s] / sum;
                for h in 0..HIDDEN_SIZE {
                    attended[t * HIDDEN_SIZE + h] += weight * v[s * HIDDEN_SIZE + h];
                }
            }
        }

        let mut new_states = vec![0.0f32; seq_len * HIDDEN_SIZE];
        for t in 0..seq_len {
            for h in 0..HIDDEN_SIZE {
                let mut sum = 0.0;
                for i in 0..HIDDEN_SIZE {
                    sum += attended[t * HIDDEN_SIZE + i] * o_w[h * HIDDEN_SIZE + i];
                }
                new_states[t * HIDDEN_SIZE + h] = sum;
            }
        }

        self.hidden_states = new_states;
        Ok(())
    }

    fn apply_ffn(&mut self) -> Result<(), Error> {
        let seq_len = self.hidden_states.len() / HIDDEN_SIZE;
        let mat_size = HIDDEN_SIZE * HIDDEN_SIZE;

        if self.current_layer_weights.len() < mat_size * 2 {
            return Err(Error::MemoryError);
        }

        let (w1, w2) = self.current_layer_weights.split_at(mat_size);

        let mut hidden = vec![0.0f32; seq_len * HIDDEN_SIZE];
        for t in 0..seq_len {
            for h in 0..HIDDEN_SIZE {
                let mut sum = 0.0;
                for i in 0..HIDDEN_SIZE {
                    sum += self.hidden_states[t * HIDDEN_SIZE + i] * w1[h * HIDDEN_SIZE + i];
                }
                hidden[t * HIDDEN_SIZE + h] = if sum > 0.0 { sum } else { 0.0 };
            }
        }

        let mut output = vec![0.0f32; seq_len * HIDDEN_SIZE];
        for t in 0..seq_len {
            for h in 0..HIDDEN_SIZE {
                let mut sum = 0.0;
                for i in 0..HIDDEN_SIZE {
                    sum += hidden[t * HIDDEN_SIZE + i] * w2[h * HIDDEN_SIZE + i];
                }
                output[t * HIDDEN_SIZE + h] = sum;
            }
        }

        self.hidden_states = output;
        Ok(())
    }
    
    fn generate_output(&self) -> Result<Vec<u32>, Error> {
        let seq_len = self.hidden_states.len() / HIDDEN_SIZE;
        if seq_len == 0 {
            return Err(Error::ComputationError);
        }
        
        let last_pos = (seq_len - 1) * HIDDEN_SIZE;
        let mut logits = Vec::with_capacity(VOCAB_SIZE);
        
        for v in 0..VOCAB_SIZE {
            let mut logit = 0.0;
            for h in 0..HIDDEN_SIZE {
                if h < self.hidden_states.len() - last_pos {
                    let weight_idx = v * HIDDEN_SIZE + h;
                    if weight_idx < self.current_layer_weights.len() {
                        logit += self.hidden_states[last_pos + h] * self.current_layer_weights[weight_idx];
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

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_layer_out_of_bounds() {
        let mut mm = MemoryManager::new();
        let mut state = ModelState::new(&mut mm);
        let res = state.load_layer_weights(LAYER_OFFSETS.len());
        assert!(matches!(res, Err(Error::MemoryError)));
    }

    #[test]
    fn apply_embeddings_invalid_token() {
        let mut mm = MemoryManager::new();
        let mut state = ModelState::new(&mut mm);
        state.current_layer_weights = vec![0.0; HIDDEN_SIZE];
        let res = state.apply_embeddings(&[VOCAB_SIZE as u32]);
        assert!(matches!(res, Err(Error::ComputationError)));
    }

    #[test]
    fn generate_output_empty_state() {
        let mut mm = MemoryManager::new();
        let state = ModelState::new(&mut mm);
        let res = state.generate_output();
        assert!(matches!(res, Err(Error::ComputationError)));
    }
}
