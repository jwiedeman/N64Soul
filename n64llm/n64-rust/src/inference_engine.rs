use alloc::vec::Vec;
use core::result::Result;
use core::fmt;
use crate::memory_manager::MemoryManager;
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
        self.load_layer_weights(0)?;
        self.apply_embeddings(input_tokens)?;
        self.unload_layer_weights();

        for layer_idx in 0..NUM_LAYERS {
            self.load_layer_weights(layer_idx * 2 + 1)?;
            self.apply_attention()?;
            self.unload_layer_weights();

            self.load_layer_weights(layer_idx * 2 + 2)?;
            self.apply_ffn()?;
            self.unload_layer_weights();
        }

        self.load_layer_weights(NUM_LAYERS * 2 + 1)?;
        let output_tokens = self.generate_output()?;
        self.unload_layer_weights();
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
        let mut new_states = Vec::with_capacity(seq_len * HIDDEN_SIZE);
        
        for i in 0..seq_len {
            for h in 0..HIDDEN_SIZE {
                let mut value = 0.0;
                for j in 0..seq_len {
                    let diff = (i as f32) - (j as f32);
                    let weight = 1.0 / (1.0 + n64_math::abs(diff));
                    value += weight * self.hidden_states[j * HIDDEN_SIZE + h];
                }
                new_states.push(value);
            }
        }
        
        self.hidden_states = new_states;
        Ok(())
    }
    
    fn apply_ffn(&mut self) -> Result<(), Error> {
        let seq_len = self.hidden_states.len() / HIDDEN_SIZE;
        let mut new_states = Vec::with_capacity(seq_len * HIDDEN_SIZE);
        
        for i in 0..self.hidden_states.len() {
            let value = if self.hidden_states[i] > 0.0 { self.hidden_states[i] } else { 0.0 };
            new_states.push(value * 0.9 + 0.1);
        }
        
        self.hidden_states = new_states;
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
