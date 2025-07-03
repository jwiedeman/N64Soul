// tokenizer.rs
// Simple tokenizer for GPT model

use alloc::string::String;
use alloc::vec::Vec;
use crate::memory_manager::MemoryManager;
use crate::n64_sys;

// Constants for tokenizer
const VOCAB_SIZE: usize = 25000;
const MAX_TOKEN_LENGTH: usize = 16;
const VOCAB_TABLE_OFFSET: u32 = 0x00E00000; // ROM offset for vocabulary data

// Simple tokenizer for GPT model
pub struct Tokenizer<'a> {
    // In a full implementation, we'd have a proper BPE tokenizer
    // For this demo, we'll use a simplified character-level approach
    memory_manager: &'a mut MemoryManager,
    // Basic vocabulary cache
    vocab_cache: Vec<(String, u32)>,
    vocab_loaded: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(memory_manager: &'a mut MemoryManager) -> Self {
        Tokenizer {
            memory_manager,
            vocab_cache: Vec::with_capacity(256), // Cache common tokens
            vocab_loaded: false,
        }
    }
    
    // Load basic vocabulary from ROM
    fn load_basic_vocab(&mut self) -> bool {
        if self.vocab_loaded {
            return true;
        }
        
        // Allocate buffer for loading vocab data
        let buffer_size = 256 * (MAX_TOKEN_LENGTH + 4); // 256 entries, each with string + u32
        let buffer_ptr = match self.memory_manager.alloc(buffer_size, 4) {
            Some(ptr) => ptr.as_ptr(),
            None => return false,
        };
        
        // Read from ROM
        unsafe {
            n64_sys::pi_read(
                buffer_ptr,
                VOCAB_TABLE_OFFSET,
                buffer_size as u32
            );
            
            // Process the loaded data
            let mut offset = 0;
            for _ in 0..256 {
                // Read token length
                let length = *buffer_ptr.add(offset) as usize;
                offset += 1;
                
                if length > 0 && length <= MAX_TOKEN_LENGTH {
                    // Read token string
                    let mut token = String::with_capacity(length);
                    for i in 0..length {
                        token.push(*buffer_ptr.add(offset + i) as char);
                    }
                    offset += MAX_TOKEN_LENGTH; // Fixed-size field
                    
                    // Read token ID
                    let token_id = u32::from_be_bytes([
                        *buffer_ptr.add(offset),
                        *buffer_ptr.add(offset + 1),
                        *buffer_ptr.add(offset + 2),
                        *buffer_ptr.add(offset + 3)
                    ]);
                    offset += 4;
                    
                    // Add to cache
                    self.vocab_cache.push((token, token_id));
                }
            }
        }
        
        self.vocab_loaded = true;
        true
    }
    
    pub fn encode(&mut self, text: &str) -> Vec<u32> {
        // This is a very simplified tokenization
        // A real implementation would use BPE tokenization

        // Ensure the basic vocabulary is loaded so cached tokens work
        self.load_basic_vocab();
        
        let mut tokens = Vec::new();
        
        // First try to match against our vocab cache
        if self.vocab_loaded {
            let mut pos = 0;
            while pos < text.len() {
                let remain = &text[pos..];
                let mut found = false;
                
                // Try to find the longest matching token in our cache
                let mut best_len = 0;
                let mut best_id = 0;
                
                for (token, id) in &self.vocab_cache {
                    if remain.starts_with(token) && token.len() > best_len {
                        best_len = token.len();
                        best_id = *id;
                        found = true;
                    }
                }
                
                if found {
                    tokens.push(best_id);
                    pos += best_len;
                } else {
                    // Fallback: character-level tokenization
                    if let Some(c) = remain.chars().next() {
                        // Encode newline explicitly
                        if c == '\n' {
                            tokens.push('\n' as u32);
                        } else {
                            tokens.push(c as u32);
                        }
                        pos += c.len_utf8();
                    } else {
                        break;
                    }
                }
            }
        } else {
            // Fallback to character-level tokenization
            for c in text.chars() {
                if c == '\n' {
                    tokens.push('\n' as u32);
                } else {
                    tokens.push(c as u32);
                }
            }
        }
        
        tokens
    }
    
    pub fn decode(&mut self, tokens: &[u32]) -> String {
        // Simple decoding

        // Ensure vocabulary is available for reverse lookup
        self.load_basic_vocab();

        let mut text = String::new();
        
        for &token in tokens {
            // First check if it's in our vocab cache
            let mut found = false;
            
            if self.vocab_loaded {
                for (tok_str, tok_id) in &self.vocab_cache {
                    if *tok_id == token {
                        text.push_str(tok_str);
                        found = true;
                        break;
                    }
                }
            }
            
            // Fallback to character-level decoding
            if !found {
                if token == ('\n' as u32) {
                    text.push('\n');
                } else if let Some(c) = core::char::from_u32(token) {
                    text.push(c);
                }
            }
        }
        
        text
    }
}