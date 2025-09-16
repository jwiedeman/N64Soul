// tokenizer.rs
// Simple tokenizer for GPT model

use crate::memory_manager::MemoryManager;
use alloc::string::String;
use alloc::vec::Vec;

const BASIC_VOCAB: &[(&str, u32)] = &[
    ("\n", 10),
    (" the", 1),
    ("and", 2),
    (" to", 3),
    ("of", 4),
    ("you", 5),
    (" I", 6),
    ("a", 7),
    ("is", 8),
    ("in", 9),
    ("for", 11),
    ("with", 12),
    ("GPT", 13),
    ("N64", 14),
    ("AI", 15),
    ("?", 63),
    ("!", 64),
    (",", 44),
    (".", 46),
    ("-", 45),
];

pub struct Tokenizer {
    vocab_cache: Vec<(String, u32)>,
    vocab_loaded: bool,
}

impl Tokenizer {
    pub fn new(_memory_manager: &mut MemoryManager) -> Self {
        Tokenizer {
            vocab_cache: Vec::with_capacity(256), // Cache common tokens
            vocab_loaded: false,
        }
    }

    // Load basic vocabulary from ROM
    fn load_basic_vocab(&mut self) -> bool {
        if self.vocab_loaded {
            return true;
        }

        for &(token, id) in BASIC_VOCAB {
            self.vocab_cache.push((String::from(token), id));
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

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_simple() {
        let mut mm = crate::memory_manager::new_for_test();
        let mut tok = Tokenizer::new(&mut mm);
        let tokens = tok.encode("hi");
        let text = tok.decode(&tokens);
        assert_eq!(text, "hi");
    }
}
