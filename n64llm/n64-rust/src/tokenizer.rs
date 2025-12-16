use crate::manifest;
use crate::memory_manager::MemoryManager;
use crate::model::names;
use crate::platform::pi;
use crate::weights;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

#[derive(Debug)]
pub enum Error {
    MissingModel,
    RomRead,
    InvalidFormat,
    Utf8,
    UnknownToken,
    InvalidToken,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingModel => write!(f, "tokenizer assets missing"),
            Error::RomRead => write!(f, "unable to read tokenizer model from ROM"),
            Error::InvalidFormat => write!(f, "invalid tokenizer model format"),
            Error::Utf8 => write!(f, "utf8 conversion error"),
            Error::UnknownToken => write!(f, "encountered unknown token"),
            Error::InvalidToken => write!(f, "token id out of range"),
        }
    }
}

pub struct Tokenizer {
    vocab: Vec<Vec<u8>>,
    token_to_id: BTreeMap<Vec<u8>, u32>,
    merges: BTreeMap<(u32, u32), (u32, u32)>,
    byte_encoder: Vec<Vec<u8>>,
    byte_decoder: BTreeMap<char, u8>,
}

impl Tokenizer {
    pub fn new(
        manifest: &manifest::Manifest,
        memory_manager: &mut MemoryManager,
    ) -> Result<Self, Error> {
        let layer = manifest
            .find(names::L_TOKENIZER_MODEL)
            .ok_or(Error::MissingModel)?;
        let mut data = vec![0u8; layer.size as usize];
        let cart_off = weights::weights_rel_to_cart_off(layer.offset as u64);
        pi::pi_dma_read(cart_off, &mut data).map_err(|_| Error::RomRead)?;
        memory_manager.log_usage("tokenizer_load");
        Self::from_bytes(&data)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, Error> {
        if data.len() < 16 || &data[0..4] != b"BPE1" {
            return Err(Error::InvalidFormat);
        }
        let version = u16::from_le_bytes([data[4], data[5]]);
        if version != 1 {
            return Err(Error::InvalidFormat);
        }
        let vocab_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
        let merge_count = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;
        let mut offset = 16usize;
        let mut vocab = Vec::with_capacity(vocab_size);
        for _ in 0..vocab_size {
            if offset + 2 > data.len() {
                return Err(Error::InvalidFormat);
            }
            let len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;
            if offset + len > data.len() {
                return Err(Error::InvalidFormat);
            }
            vocab.push(data[offset..offset + len].to_vec());
            offset += len;
        }

        let mut merges = BTreeMap::new();
        for rank in 0..merge_count {
            if offset + 12 > data.len() {
                return Err(Error::InvalidFormat);
            }
            let left = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            let right = u32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            let result = u32::from_le_bytes([
                data[offset + 8],
                data[offset + 9],
                data[offset + 10],
                data[offset + 11],
            ]);
            merges.insert((left, right), (result, rank as u32));
            offset += 12;
        }

        let mut token_to_id = BTreeMap::new();
        for (idx, token) in vocab.iter().enumerate() {
            token_to_id.insert(token.clone(), idx as u32);
        }

        let (byte_encoder, byte_decoder) = build_byte_maps();

        Ok(Tokenizer {
            vocab,
            token_to_id,
            merges,
            byte_encoder,
            byte_decoder,
        })
    }

    pub fn encode(&self, text: &str) -> Result<Vec<u32>, Error> {
        let mut output = Vec::new();
        for piece in pre_tokenize(text) {
            let ids = self.encode_piece(&piece)?;
            output.extend(ids);
        }
        Ok(output)
    }

    pub fn decode(&self, tokens: &[u32]) -> Result<String, Error> {
        let mut bytes = Vec::new();
        for &token in tokens {
            let entry = self.vocab.get(token as usize).ok_or(Error::InvalidToken)?;
            let s = core::str::from_utf8(entry).map_err(|_| Error::Utf8)?;
            for ch in s.chars() {
                let b = *self.byte_decoder.get(&ch).ok_or(Error::InvalidToken)?;
                bytes.push(b);
            }
        }
        String::from_utf8(bytes).map_err(|_| Error::Utf8)
    }

    fn encode_piece(&self, piece: &str) -> Result<Vec<u32>, Error> {
        let mut ids = Vec::new();
        for &b in piece.as_bytes() {
            let encoded = &self.byte_encoder[b as usize];
            let id = self.token_to_id.get(encoded).ok_or(Error::UnknownToken)?;
            ids.push(*id);
        }
        Ok(self.merge_ids(ids))
    }

    fn merge_ids(&self, mut ids: Vec<u32>) -> Vec<u32> {
        if ids.len() <= 1 {
            return ids;
        }
        loop {
            let mut best_rank = u32::MAX;
            let mut best_index = None;
            let mut best_result = 0u32;
            for i in 0..ids.len() - 1 {
                let pair = (ids[i], ids[i + 1]);
                if let Some(&(result, rank)) = self.merges.get(&pair) {
                    if rank < best_rank {
                        best_rank = rank;
                        best_index = Some(i);
                        best_result = result;
                    }
                }
            }
            if let Some(idx) = best_index {
                ids[idx] = best_result;
                ids.remove(idx + 1);
            } else {
                break;
            }
        }
        ids
    }
}

fn build_byte_maps() -> (Vec<Vec<u8>>, BTreeMap<char, u8>) {
    let mut bs: Vec<u32> = (b'!'..=b'~').map(|b| b as u32).collect();
    bs.extend((0xA1u32)..=0xACu32);
    bs.extend((0xAEu32)..=0xFFu32);
    let mut cs = bs.clone();
    let mut n = 0u32;
    for b in 0u32..=255 {
        if !bs.contains(&b) {
            bs.push(b);
            cs.push(256 + n);
            n += 1;
        }
    }
    let mut encoder = vec![Vec::new(); 256];
    let mut decoder = BTreeMap::new();
    for (b, c) in bs.into_iter().zip(cs.into_iter()) {
        if let Some(ch) = char::from_u32(c) {
            let mut buf = String::new();
            buf.push(ch);
            encoder[b as usize] = buf.as_bytes().to_vec();
            decoder.insert(ch, b as u8);
        }
    }
    (encoder, decoder)
}

fn pre_tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '\'' {
            let suffixes = ["'s", "'t", "'re", "'ve", "'m", "'ll", "'d"];
            let mut matched = false;
            for suf in &suffixes {
                let end = i + suf.chars().count();
                if end <= chars.len() && chars[i..end].iter().collect::<String>() == *suf {
                    tokens.push(suf.to_string());
                    i = end;
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
        }

        if chars[i].is_whitespace() {
            let mut j = i;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            tokens.push(chars[i..j].iter().collect());
            i = j;
            continue;
        }

        let start = i;
        let mut has_space_prefix = false;
        if chars[i] == ' ' {
            has_space_prefix = true;
            i += 1;
        }

        if i < chars.len() && chars[i].is_alphabetic() {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_alphabetic() {
                j += 1;
            }
            tokens.push(chars[start..j].iter().collect());
            i = j;
            continue;
        }

        if i < chars.len() && chars[i].is_numeric() {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_numeric() {
                j += 1;
            }
            tokens.push(chars[start..j].iter().collect());
            i = j;
            continue;
        }

        if i < chars.len() {
            let mut j = i + 1;
            while j < chars.len()
                && !chars[j].is_whitespace()
                && !chars[j].is_alphabetic()
                && !chars[j].is_numeric()
            {
                j += 1;
            }
            let token = if has_space_prefix {
                chars[start..j].iter().collect()
            } else {
                chars[i..j].iter().collect()
            };
            tokens.push(token);
            i = j;
            continue;
        }

        tokens.push(chars[i].to_string());
        i += 1;
    }
    tokens
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_model() -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"BPE1");
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&3u32.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes());
        let tokens = [b"a", b"b", b"ab"];
        for t in &tokens {
            data.extend_from_slice(&(t.len() as u16).to_le_bytes());
            data.extend_from_slice(t);
        }
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&2u32.to_le_bytes());
        data
    }

    #[test]
    fn encode_merge() {
        let model = Tokenizer::from_bytes(&build_test_model()).unwrap();
        let tokens = model.encode("ab").unwrap();
        assert_eq!(tokens, vec![2]);
    }

    #[test]
    fn decode_roundtrip() {
        let model = Tokenizer::from_bytes(&build_test_model()).unwrap();
        let text = model.decode(&[0, 1]).unwrap();
        assert_eq!(text, "ab");
    }
}
