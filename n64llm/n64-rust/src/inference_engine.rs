use crate::display;
use crate::io::rom_reader::FlatRomReader;
use crate::manifest;
use crate::memory_manager::MemoryManager;
use crate::model::names;
use crate::n64_math;
use crate::stream::streamer::stream_entry;
use crate::{platform::pi, weights};
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp;
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

const MAX_SEQ_LENGTH: usize = 128;
const LAYER_NORM_EPS: f32 = 1e-5;

pub struct ModelState<'a> {
    memory_manager: &'a mut MemoryManager,
    manifest: &'a manifest::Manifest,
    dims: crate::model::dims::ModelDims,
    plan: LayerPlan,
    hidden_states: Vec<f32>,
    residual: Vec<f32>,
    normed: Vec<f32>,
    attn_output: Vec<f32>,
    context: Vec<f32>,
    ffn_hidden: Vec<f32>,
    ffn_mid: Vec<f32>,
    weights_a: Vec<f32>,
    bias_a: Vec<f32>,
    bias_b: Vec<f32>,
    qkv_buffer: Vec<f32>,
    scores: Vec<f32>,
    token_row: Vec<f32>,
    dma_buffer: Vec<u8>,
}

impl<'a> ModelState<'a> {
    pub fn new(memory_manager: &'a mut MemoryManager, manifest: &'a manifest::Manifest) -> Self {
        let dims = manifest.dims;
        let plan = LayerPlan::from_manifest(manifest);
        ModelState {
            memory_manager,
            manifest,
            dims,
            plan,
            hidden_states: Vec::new(),
            residual: Vec::new(),
            normed: Vec::new(),
            attn_output: Vec::new(),
            context: Vec::new(),
            ffn_hidden: Vec::new(),
            ffn_mid: Vec::new(),
            weights_a: Vec::new(),
            bias_a: Vec::new(),
            bias_b: Vec::new(),
            qkv_buffer: Vec::new(),
            scores: Vec::new(),
            token_row: Vec::new(),
            dma_buffer: Vec::new(),
        }
    }

    pub fn run_inference(&mut self, input_tokens: &[u32]) -> Result<Vec<u32>, Error> {
        if input_tokens.is_empty() || input_tokens.len() > MAX_SEQ_LENGTH {
            return Err(Error::ComputationError);
        }

        let hidden_size = self.dims.d_model as usize;
        let seq_len = input_tokens.len();
        ensure_vec(&mut self.hidden_states, seq_len * hidden_size);
        ensure_vec(&mut self.residual, seq_len * hidden_size);
        ensure_vec(&mut self.normed, seq_len * hidden_size);
        ensure_vec(&mut self.attn_output, seq_len * hidden_size);
        ensure_vec(&mut self.ffn_hidden, seq_len * hidden_size);

        let num_layers = self.plan.layers.len();
        display::show_progress(0, num_layers + 2);
        self.apply_embeddings(input_tokens)?;

        // Process layers by index to avoid borrow conflicts
        for idx in 0..num_layers {
            display::show_progress(idx + 1, num_layers + 2);
            self.process_layer_by_index(idx, seq_len, hidden_size)?;
            self.memory_manager.log_usage("layer");
        }

        self.apply_final_norm(hidden_size)?;
        display::show_progress(num_layers + 1, num_layers + 2);
        let output_tokens = self.generate_output(seq_len, hidden_size)?;
        display::show_progress(num_layers + 2, num_layers + 2);
        Ok(output_tokens)
    }

    fn process_layer_by_index(
        &mut self,
        layer_idx: usize,
        seq_len: usize,
        hidden_size: usize,
    ) -> Result<(), Error> {
        // Copy layer spec to avoid borrow conflicts
        let layer = self.plan.layers[layer_idx];
        self.process_layer(&layer, seq_len, hidden_size)
    }

    fn process_layer(
        &mut self,
        layer: &LayerSpec,
        seq_len: usize,
        hidden_size: usize,
    ) -> Result<(), Error> {
        self.residual.copy_from_slice(&self.hidden_states);

        load_layer_f32(self.manifest, layer.ln1_weight, &mut self.dma_buffer, &mut self.weights_a)?;
        load_layer_f32(self.manifest, layer.ln1_bias, &mut self.dma_buffer, &mut self.bias_a)?;
        layer_norm(
            &self.hidden_states,
            &self.weights_a,
            &self.bias_a,
            hidden_size,
            &mut self.normed,
        )?;

        self.apply_attention(layer, seq_len, hidden_size)?;

        for i in 0..self.hidden_states.len() {
            self.hidden_states[i] = self.attn_output[i] + self.residual[i];
        }

        self.residual.copy_from_slice(&self.hidden_states);

        load_layer_f32(self.manifest, layer.ln2_weight, &mut self.dma_buffer, &mut self.weights_a)?;
        load_layer_f32(self.manifest, layer.ln2_bias, &mut self.dma_buffer, &mut self.bias_a)?;
        layer_norm(
            &self.hidden_states,
            &self.weights_a,
            &self.bias_a,
            hidden_size,
            &mut self.normed,
        )?;

        self.apply_ffn(layer, seq_len, hidden_size)?;

        for i in 0..self.hidden_states.len() {
            self.hidden_states[i] = self.ffn_hidden[i] + self.residual[i];
        }

        Ok(())
    }

    fn apply_embeddings(&mut self, input_tokens: &[u32]) -> Result<(), Error> {
        let embedding_idx = self
            .plan
            .embedding
            .ok_or(Error::MissingLayer(names::L_TOK_EMB))?;
        let pos_idx = self
            .plan
            .positional
            .ok_or(Error::MissingLayer(names::L_POS_EMB))?;

        let hidden_size = self.dims.d_model as usize;
        ensure_vec(&mut self.token_row, hidden_size);

        for (pos, &token) in input_tokens.iter().enumerate() {
            if token >= self.dims.vocab_size {
                return Err(Error::ComputationError);
            }
            read_matrix_row(self.manifest, self.dims, embedding_idx, token, &mut self.dma_buffer, &mut self.token_row)?;
            let start = pos * hidden_size;
            self.hidden_states[start..start + hidden_size].copy_from_slice(&self.token_row);

            let pos_id = cmp::min(pos as u32, self.dims.n_positions.saturating_sub(1));
            read_matrix_row(self.manifest, self.dims, pos_idx, pos_id, &mut self.dma_buffer, &mut self.token_row)?;
            for i in 0..hidden_size {
                self.hidden_states[start + i] += self.token_row[i];
            }
        }

        Ok(())
    }

    fn apply_attention(
        &mut self,
        layer: &LayerSpec,
        seq_len: usize,
        hidden_size: usize,
    ) -> Result<(), Error> {
        let n_heads = self.dims.n_head as usize;
        if n_heads == 0 || hidden_size % n_heads != 0 {
            return Err(Error::ComputationError);
        }
        let head_dim = hidden_size / n_heads;
        let three_hidden = hidden_size * 3;

        load_layer_f32(self.manifest, layer.attn_bias, &mut self.dma_buffer, &mut self.bias_a)?;
        if self.bias_a.len() != three_hidden {
            return Err(Error::ComputationError);
        }

        stream_layer_matmul(
            self.manifest,
            layer.attn_weight,
            &self.normed,
            seq_len,
            hidden_size,
            three_hidden,
            &self.bias_a,
            &mut self.qkv_buffer,
        )?;

        ensure_vec(&mut self.context, seq_len * hidden_size);
        for val in self.context.iter_mut() {
            *val = 0.0;
        }
        ensure_vec(&mut self.scores, seq_len);
        for val in self.scores.iter_mut() {
            *val = 0.0;
        }

        for head in 0..n_heads {
            for t in 0..seq_len {
                for s in 0..seq_len {
                    let mut dot = 0.0f32;
                    for i in 0..head_dim {
                        let q = self.qkv_buffer[t * three_hidden + head * head_dim + i];
                        let k =
                            self.qkv_buffer[s * three_hidden + hidden_size + head * head_dim + i];
                        dot += q * k;
                    }
                    self.scores[s] = dot / n64_math::sqrt(head_dim as f32);
                }
                let mut max_score = f32::NEG_INFINITY;
                for &v in &self.scores {
                    if v > max_score {
                        max_score = v;
                    }
                }
                let mut sum = 0.0f32;
                for val in self.scores.iter_mut() {
                    *val = n64_math::exp_approx(*val - max_score);
                    sum += *val;
                }
                if sum == 0.0 {
                    return Err(Error::ComputationError);
                }
                for s in 0..seq_len {
                    let weight = self.scores[s] / sum;
                    for i in 0..head_dim {
                        let v = self.qkv_buffer
                            [s * three_hidden + 2 * hidden_size + head * head_dim + i];
                        self.context[t * hidden_size + head * head_dim + i] += weight * v;
                    }
                }
            }
        }

        load_layer_f32(self.manifest, layer.attn_proj_bias, &mut self.dma_buffer, &mut self.bias_b)?;
        if self.bias_b.len() != hidden_size {
            return Err(Error::ComputationError);
        }

        stream_layer_matmul(
            self.manifest,
            layer.attn_proj_weight,
            &self.context,
            seq_len,
            hidden_size,
            hidden_size,
            &self.bias_b,
            &mut self.attn_output,
        )?;

        Ok(())
    }

    fn apply_ffn(
        &mut self,
        layer: &LayerSpec,
        seq_len: usize,
        hidden_size: usize,
    ) -> Result<(), Error> {
        let d_ff = self.dims.d_ff as usize;
        if d_ff == 0 {
            return Err(Error::ComputationError);
        }

        load_layer_f32(self.manifest, layer.ffn_bias, &mut self.dma_buffer, &mut self.bias_a)?;
        if self.bias_a.len() != d_ff {
            return Err(Error::ComputationError);
        }

        stream_layer_matmul(
            self.manifest,
            layer.ffn_weight,
            &self.normed,
            seq_len,
            hidden_size,
            d_ff,
            &self.bias_a,
            &mut self.ffn_mid,
        )?;

        for val in self.ffn_mid.iter_mut() {
            *val = gelu(*val);
        }

        load_layer_f32(self.manifest, layer.ffn_proj_bias, &mut self.dma_buffer, &mut self.bias_b)?;
        if self.bias_b.len() != hidden_size {
            return Err(Error::ComputationError);
        }

        stream_layer_matmul(
            self.manifest,
            layer.ffn_proj_weight,
            &self.ffn_mid,
            seq_len,
            d_ff,
            hidden_size,
            &self.bias_b,
            &mut self.ffn_hidden,
        )?;

        Ok(())
    }

    fn apply_final_norm(&mut self, hidden_size: usize) -> Result<(), Error> {
        let weight_idx = self
            .plan
            .final_norm_weight
            .ok_or(Error::MissingLayer(names::L_FINAL_NORM_WEIGHT))?;
        let bias_idx = self
            .plan
            .final_norm_bias
            .ok_or(Error::MissingLayer(names::L_FINAL_NORM_BIAS))?;

        load_layer_f32(self.manifest, weight_idx, &mut self.dma_buffer, &mut self.weights_a)?;
        load_layer_f32(self.manifest, bias_idx, &mut self.dma_buffer, &mut self.bias_a)?;
        layer_norm(
            &self.hidden_states,
            &self.weights_a,
            &self.bias_a,
            hidden_size,
            &mut self.normed,
        )?;
        self.hidden_states.copy_from_slice(&self.normed);

        // Keep buffers sized for downstream output projection.
        ensure_vec(&mut self.scores, self.dims.vocab_size as usize);
        ensure_vec(&mut self.token_row, hidden_size);

        Ok(())
    }

    fn generate_output(&mut self, seq_len: usize, hidden_size: usize) -> Result<Vec<u32>, Error> {
        if seq_len == 0 {
            return Err(Error::ComputationError);
        }
        let output_idx = self
            .plan
            .output
            .ok_or(Error::MissingLayer(names::L_LM_HEAD))?;

        let vocab_size = self.dims.vocab_size as usize;
        ensure_vec(&mut self.scores, vocab_size);

        // Copy last hidden state to avoid borrow conflict
        let last_offset = (seq_len - 1) * hidden_size;
        let mut last_hidden_copy = vec![0.0f32; hidden_size];
        last_hidden_copy.copy_from_slice(&self.hidden_states[last_offset..last_offset + hidden_size]);

        for token_id in 0..vocab_size {
            read_matrix_row(self.manifest, self.dims, output_idx, token_id as u32, &mut self.dma_buffer, &mut self.token_row)?;
            let mut sum = 0.0f32;
            for i in 0..hidden_size {
                sum += self.token_row[i] * last_hidden_copy[i];
            }
            self.scores[token_id] = sum;
        }

        let mut max_logit = f32::NEG_INFINITY;
        for &v in &self.scores {
            if v > max_logit {
                max_logit = v;
            }
        }
        let mut total = 0.0f32;
        for val in self.scores.iter_mut() {
            *val = n64_math::exp_approx(*val - max_logit);
            total += *val;
        }
        if total == 0.0 {
            return Err(Error::ComputationError);
        }

        let mut best_idx = 0usize;
        let mut best_prob = -1.0f32;
        for (idx, prob) in self.scores.iter_mut().enumerate() {
            *prob /= total;
            if *prob > best_prob {
                best_prob = *prob;
                best_idx = idx;
            }
        }

        Ok(vec![best_idx as u32])
    }
}

fn ensure_vec(vec: &mut Vec<f32>, len: usize) {
    if vec.len() != len {
        vec.resize(len, 0.0);
    }
}

fn load_layer_f32(
    manifest: &manifest::Manifest,
    idx: usize,
    dma_buffer: &mut Vec<u8>,
    out: &mut Vec<f32>,
) -> Result<(), Error> {
    let layer = manifest.layers.get(idx).ok_or(Error::MemoryError)?;
    let size = layer.size as usize;
    dma_buffer.resize(size, 0);
    let cart_off = weights::weights_rel_to_cart_off(layer.offset as u64);
    pi::pi_dma_read(cart_off, dma_buffer).map_err(|_| Error::RomReadError)?;

    if dma_buffer.len() % 4 != 0 {
        return Err(Error::ComputationError);
    }
    let count = dma_buffer.len() / 4;
    out.resize(count, 0.0);
    for (i, chunk) in dma_buffer.chunks_exact(4).enumerate() {
        out[i] = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    Ok(())
}

fn read_matrix_row(
    manifest: &manifest::Manifest,
    dims: crate::model::dims::ModelDims,
    idx: usize,
    row: u32,
    dma_buffer: &mut Vec<u8>,
    out: &mut Vec<f32>,
) -> Result<(), Error> {
    let layer = manifest.layers.get(idx).ok_or(Error::MemoryError)?;
    let hidden_size = dims.d_model as usize;
    let row_bytes = hidden_size * 4;
    if row_bytes == 0 || layer.size as usize % row_bytes != 0 {
        return Err(Error::ComputationError);
    }
    let rows = layer.size as usize / row_bytes;
    if row as usize >= rows {
        return Err(Error::ComputationError);
    }
    dma_buffer.resize(row_bytes, 0);
    let offset = layer.offset as u64 + (row as u64) * row_bytes as u64;
    let cart_off = weights::weights_rel_to_cart_off(offset);
    pi::pi_dma_read(cart_off, dma_buffer).map_err(|_| Error::RomReadError)?;
    out.resize(hidden_size, 0.0);
    for (i, chunk) in dma_buffer.chunks_exact(4).enumerate() {
        out[i] = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    Ok(())
}

fn stream_layer_matmul(
    manifest: &manifest::Manifest,
    idx: usize,
    input: &[f32],
    seq_len: usize,
    in_dim: usize,
    out_dim: usize,
    bias: &[f32],
    output: &mut Vec<f32>,
) -> Result<(), Error> {
    if input.len() != seq_len * in_dim {
        return Err(Error::ComputationError);
    }
    if bias.len() != out_dim {
        return Err(Error::ComputationError);
    }

    let layer = manifest.layers.get(idx).ok_or(Error::MemoryError)?;
    let expected_bytes = (in_dim as u64)
        .checked_mul(out_dim as u64)
        .and_then(|v| v.checked_mul(core::mem::size_of::<f32>() as u64))
        .ok_or(Error::ComputationError)?;
    if expected_bytes != layer.size as u64 {
        return Err(Error::ComputationError);
    }

    output.resize(seq_len * out_dim, 0.0);
    for chunk in output.chunks_mut(out_dim) {
        chunk.copy_from_slice(bias);
    }

    if expected_bytes == 0 {
        return Ok(());
    }

    let cart_off = weights::weights_rel_to_cart_off(layer.offset as u64);
    let mut rr = FlatRomReader::new();
    let mut row = 0usize;
    let mut col = 0usize;
    let mut compute_ok = true;

    let stats = stream_entry(&mut rr, cart_off, expected_bytes, |chunk| {
        if !compute_ok {
            return;
        }
        let mut iter = chunk.chunks_exact(4);
        for word in iter.by_ref() {
            if row >= in_dim {
                compute_ok = false;
                break;
            }
            let weight = f32::from_le_bytes([word[0], word[1], word[2], word[3]]);
            for t in 0..seq_len {
                let inp = input[t * in_dim + row];
                let out_idx = t * out_dim + col;
                output[out_idx] += inp * weight;
            }
            col += 1;
            if col == out_dim {
                col = 0;
                row += 1;
            }
        }
        if !iter.remainder().is_empty() {
            compute_ok = false;
        }
    });

    if stats.is_none() {
        return Err(Error::RomReadError);
    }

    if !compute_ok || row != in_dim || col != 0 {
        return Err(Error::ComputationError);
    }

    Ok(())
}

fn layer_norm(
    input: &[f32],
    gamma: &[f32],
    beta: &[f32],
    hidden_size: usize,
    out: &mut Vec<f32>,
) -> Result<(), Error> {
    if gamma.len() != hidden_size || beta.len() != hidden_size {
        return Err(Error::ComputationError);
    }
    if input.len() % hidden_size != 0 {
        return Err(Error::ComputationError);
    }
    let seq_len = input.len() / hidden_size;
    out.resize(input.len(), 0.0);

    for t in 0..seq_len {
        let start = t * hidden_size;
        let slice = &input[start..start + hidden_size];
        let mut mean = 0.0f32;
        for &x in slice {
            mean += x;
        }
        mean /= hidden_size as f32;
        let mut var = 0.0f32;
        for &x in slice {
            let diff = x - mean;
            var += diff * diff;
        }
        var /= hidden_size as f32;
        let inv_std = 1.0f32 / n64_math::sqrt(var + LAYER_NORM_EPS);
        for i in 0..hidden_size {
            let norm = (slice[i] - mean) * inv_std;
            out[start + i] = norm * gamma[i] + beta[i];
        }
    }

    Ok(())
}

fn gelu(x: f32) -> f32 {
    let c = 0.7978845608f32; // sqrt(2/pi)
    let inner = c * (x + 0.044_715f32 * x * x * x);
    let e2x = n64_math::exp_approx(2.0 * inner);
    let tanh = (e2x - 1.0) / (e2x + 1.0);
    0.5f32 * x * (1.0 + tanh)
}

#[derive(Copy, Clone)]
struct LayerSpec {
    ln1_weight: usize,
    ln1_bias: usize,
    attn_weight: usize,
    attn_bias: usize,
    attn_proj_weight: usize,
    attn_proj_bias: usize,
    ln2_weight: usize,
    ln2_bias: usize,
    ffn_weight: usize,
    ffn_bias: usize,
    ffn_proj_weight: usize,
    ffn_proj_bias: usize,
}

struct LayerPlan {
    embedding: Option<usize>,
    positional: Option<usize>,
    output: Option<usize>,
    final_norm_weight: Option<usize>,
    final_norm_bias: Option<usize>,
    layers: Vec<LayerSpec>,
}

impl LayerPlan {
    fn from_manifest(manifest: &manifest::Manifest) -> Self {
        let mut embedding = None;
        let mut positional = None;
        let mut output = None;
        let mut final_norm_weight = None;
        let mut final_norm_bias = None;
        let mut layer_map: BTreeMap<usize, PartialLayer> = BTreeMap::new();

        for (idx, layer) in manifest.layers.iter().enumerate() {
            match layer.name.as_str() {
                names::L_TOK_EMB => embedding = Some(idx),
                names::L_POS_EMB => positional = Some(idx),
                names::L_LM_HEAD => output = Some(idx),
                names::L_FINAL_NORM_WEIGHT => final_norm_weight = Some(idx),
                names::L_FINAL_NORM_BIAS => final_norm_bias = Some(idx),
                _ => {
                    if let Some((layer_idx, field)) = parse_layer_entry(&layer.name) {
                        layer_map
                            .entry(layer_idx)
                            .or_insert_with(PartialLayer::default)
                            .set(field, idx);
                    }
                }
            }
        }

        let mut layers = Vec::new();
        for (_, partial) in layer_map.into_iter() {
            if let Some(spec) = partial.into_spec() {
                layers.push(spec);
            }
        }

        LayerPlan {
            embedding,
            positional,
            output,
            final_norm_weight,
            final_norm_bias,
            layers,
        }
    }
}

#[derive(Default)]
struct PartialLayer {
    ln1_weight: Option<usize>,
    ln1_bias: Option<usize>,
    attn_weight: Option<usize>,
    attn_bias: Option<usize>,
    attn_proj_weight: Option<usize>,
    attn_proj_bias: Option<usize>,
    ln2_weight: Option<usize>,
    ln2_bias: Option<usize>,
    ffn_weight: Option<usize>,
    ffn_bias: Option<usize>,
    ffn_proj_weight: Option<usize>,
    ffn_proj_bias: Option<usize>,
}

impl PartialLayer {
    fn set(&mut self, field: LayerField, idx: usize) {
        match field {
            LayerField::Ln1Weight => self.ln1_weight = Some(idx),
            LayerField::Ln1Bias => self.ln1_bias = Some(idx),
            LayerField::AttnWeight => self.attn_weight = Some(idx),
            LayerField::AttnBias => self.attn_bias = Some(idx),
            LayerField::AttnProjWeight => self.attn_proj_weight = Some(idx),
            LayerField::AttnProjBias => self.attn_proj_bias = Some(idx),
            LayerField::Ln2Weight => self.ln2_weight = Some(idx),
            LayerField::Ln2Bias => self.ln2_bias = Some(idx),
            LayerField::FfnWeight => self.ffn_weight = Some(idx),
            LayerField::FfnBias => self.ffn_bias = Some(idx),
            LayerField::FfnProjWeight => self.ffn_proj_weight = Some(idx),
            LayerField::FfnProjBias => self.ffn_proj_bias = Some(idx),
        }
    }

    fn into_spec(self) -> Option<LayerSpec> {
        Some(LayerSpec {
            ln1_weight: self.ln1_weight?,
            ln1_bias: self.ln1_bias?,
            attn_weight: self.attn_weight?,
            attn_bias: self.attn_bias?,
            attn_proj_weight: self.attn_proj_weight?,
            attn_proj_bias: self.attn_proj_bias?,
            ln2_weight: self.ln2_weight?,
            ln2_bias: self.ln2_bias?,
            ffn_weight: self.ffn_weight?,
            ffn_bias: self.ffn_bias?,
            ffn_proj_weight: self.ffn_proj_weight?,
            ffn_proj_bias: self.ffn_proj_bias?,
        })
    }
}

#[derive(Copy, Clone)]
enum LayerField {
    Ln1Weight,
    Ln1Bias,
    AttnWeight,
    AttnBias,
    AttnProjWeight,
    AttnProjBias,
    Ln2Weight,
    Ln2Bias,
    FfnWeight,
    FfnBias,
    FfnProjWeight,
    FfnProjBias,
}

fn parse_layer_entry(name: &str) -> Option<(usize, LayerField)> {
    if !name.starts_with("layer") {
        return None;
    }
    let bytes = name.as_bytes();
    let mut pos = 5usize;
    if pos >= bytes.len() {
        return None;
    }
    let mut idx = 0usize;
    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
        idx = idx * 10 + (bytes[pos] - b'0') as usize;
        pos += 1;
    }
    if pos >= bytes.len() || bytes[pos] != b'.' {
        return None;
    }
    pos += 1;
    let suffix = &name[pos..];
    let field = match suffix {
        "ln1.weight" => LayerField::Ln1Weight,
        "ln1.bias" => LayerField::Ln1Bias,
        "attn.qkv.weight" => LayerField::AttnWeight,
        "attn.qkv.bias" => LayerField::AttnBias,
        "attn.proj.weight" => LayerField::AttnProjWeight,
        "attn.proj.bias" => LayerField::AttnProjBias,
        "ln2.weight" => LayerField::Ln2Weight,
        "ln2.bias" => LayerField::Ln2Bias,
        "ffn.in.weight" => LayerField::FfnWeight,
        "ffn.in.bias" => LayerField::FfnBias,
        "ffn.out.weight" => LayerField::FfnProjWeight,
        "ffn.out.bias" => LayerField::FfnProjBias,
        _ => return None,
    };
    Some((idx, field))
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn parse_layer_fields() {
        assert!(matches!(
            parse_layer_entry("layer00.ln1.weight"),
            Some((0, LayerField::Ln1Weight))
        ));
        assert!(parse_layer_entry("layer03.ffn.out.bias").is_some());
        assert!(parse_layer_entry("tok_embeddings").is_none());
    }

    #[test]
    fn layer_plan_collects_layers() {
        let mut manifest = manifest::Manifest {
            layers: Vec::new(),
            align: 64,
            dims: crate::model::dims::ModelDims::fallback(),
        };
        let names = [
            names::L_TOK_EMB,
            names::L_POS_EMB,
            "layer0.ln1.weight",
            "layer0.ln1.bias",
            "layer0.attn.qkv.weight",
            "layer0.attn.qkv.bias",
            "layer0.attn.proj.weight",
            "layer0.attn.proj.bias",
            "layer0.ln2.weight",
            "layer0.ln2.bias",
            "layer0.ffn.in.weight",
            "layer0.ffn.in.bias",
            "layer0.ffn.out.weight",
            "layer0.ffn.out.bias",
            names::L_FINAL_NORM_WEIGHT,
            names::L_FINAL_NORM_BIAS,
            names::L_LM_HEAD,
        ];
        for (i, name) in names.iter().enumerate() {
            manifest.layers.push(manifest::Layer {
                name: name.to_string(),
                offset: (i * 16) as u32,
                size: 16,
            });
        }

        let plan = LayerPlan::from_manifest(&manifest);
        assert_eq!(plan.layers.len(), 1);
        assert!(plan.embedding.is_some());
        assert!(plan.positional.is_some());
        assert!(plan.output.is_some());
        assert!(plan.final_norm_weight.is_some());
        assert!(plan.final_norm_bias.is_some());
    }
}
