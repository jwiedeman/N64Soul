import torch
from transformers import GPT2LMHeadModel, GPT2Tokenizer
import numpy as np

# Configuration for the reduced model expected by the N64 inference engine.
MODEL_NAME = "distilgpt2"
OUTPUT_FILENAME = "n64_model_weights_reduced.bin"

VOCAB_SIZE = 25000   # Reduced vocabulary size
HIDDEN_SIZE = 384    # Reduced hidden dimension
NUM_LAYERS = 6       # Number of transformer layers

print("🔹 Loading model and tokenizer...")
tokenizer = GPT2Tokenizer.from_pretrained(MODEL_NAME)
model = GPT2LMHeadModel.from_pretrained(MODEL_NAME)

# Convert model to FP16 precision for efficiency.
print("🔹 Converting model to FP16 precision...")
model = model.half()
model.eval()

def tensor_to_array(tensor: torch.Tensor) -> np.ndarray:
    """
    Converts the given tensor to a flattened numpy array of FP16.
    """
    return tensor.detach().flatten().cpu().numpy().astype(np.float16)

segments = []

# Segment 0: Reduced Embedding layer.
# Original shape: [original_vocab, original_hidden] (typically [50257, 768]).
# Reduced to: [VOCAB_SIZE, HIDDEN_SIZE] = [25000, 384].
print("🔹 Processing reduced embedding layer...")
emb = model.transformer.wte.weight  # shape: [orig_vocab, orig_hidden]
emb_reduced = emb[:VOCAB_SIZE, :HIDDEN_SIZE]
segments.append(tensor_to_array(emb_reduced))
print(f"    Embedding reduced size: {segments[-1].size} floats")

# Process each transformer layer.
for i in range(NUM_LAYERS):
    print(f"🔹 Processing transformer layer {i}...")
    layer = model.transformer.h[i]
    
    # Attention segment:
    # For GPT-2, the self-attention module has a combined linear layer "c_attn"
    # with shape [orig_hidden, 3*orig_hidden] and a projection "c_proj" with shape [orig_hidden, orig_hidden].
    # We reduce these by taking the first HIDDEN_SIZE rows and
    # first 3*HIDDEN_SIZE columns for c_attn and first HIDDEN_SIZE columns for c_proj.
    attn_c_attn = layer.attn.c_attn.weight  # shape: [orig_hidden, 3*orig_hidden]
    attn_c_attn_reduced = attn_c_attn[:HIDDEN_SIZE, :3*HIDDEN_SIZE]  # becomes [384, 1152]
    
    attn_c_proj = layer.attn.c_proj.weight  # shape: [orig_hidden, orig_hidden]
    attn_c_proj_reduced = attn_c_proj[:HIDDEN_SIZE, :HIDDEN_SIZE]   # becomes [384, 384]
    
    # Concatenate the reduced attention weights.
    attn_weights = torch.cat([attn_c_attn_reduced.flatten(), attn_c_proj_reduced.flatten()])
    segments.append(tensor_to_array(attn_weights))
    print(f"    Attention weights reduced size: {segments[-1].size} floats")
    
    # FFN segment:
    # For GPT-2, the FFN consists of mlp.c_fc (shape: [orig_hidden, 4*orig_hidden]) and mlp.c_proj (shape: [orig_hidden, orig_hidden]).
    # We reduce these by taking:
    #   mlp.c_fc -> first HIDDEN_SIZE rows, first 4*HIDDEN_SIZE columns (i.e. [384, 1536])
    #   mlp.c_proj -> [HIDDEN_SIZE, HIDDEN_SIZE]
    mlp_c_fc = layer.mlp.c_fc.weight  # shape: [orig_hidden, 4*orig_hidden]
    mlp_c_fc_reduced = mlp_c_fc[:HIDDEN_SIZE, :4*HIDDEN_SIZE]  # becomes [384, 1536]
    
    mlp_c_proj = layer.mlp.c_proj.weight  # shape: [orig_hidden, orig_hidden]
    mlp_c_proj_reduced = mlp_c_proj[:HIDDEN_SIZE, :HIDDEN_SIZE]  # becomes [384, 384]
    
    ffn_weights = torch.cat([mlp_c_fc_reduced.flatten(), mlp_c_proj_reduced.flatten()])
    segments.append(tensor_to_array(ffn_weights))
    print(f"    FFN weights reduced size: {segments[-1].size} floats")

# Segment 13: Reduced Output layer.
# For GPT-2, lm_head has shape [orig_vocab, orig_hidden]; reduce to [VOCAB_SIZE, HIDDEN_SIZE].
print("🔹 Processing reduced output layer...")
out = model.lm_head.weight  # shape: [orig_vocab, orig_hidden]
out_reduced = out[:VOCAB_SIZE, :HIDDEN_SIZE]
segments.append(tensor_to_array(out_reduced))
print(f"    Output layer reduced size: {segments[-1].size} floats")

# We expect 1 + 6*2 + 1 = 14 segments.
num_segments = len(segments)
print(f"🔹 Total segments: {num_segments}")

# Write out the data to a binary file with a simple header format.
# Format:
#   int32 num_segments
#   For each segment:
#       int32 segment_length (number of FP16 values)
#       segment_length * 2 bytes of FP16 data
print("🔹 Writing binary file...")
with open(OUTPUT_FILENAME, "wb") as f:
    f.write(num_segments.to_bytes(4, byteorder="little", signed=True))
    for idx, seg in enumerate(segments):
        seg_length = seg.size
        f.write(seg_length.to_bytes(4, byteorder="little", signed=True))
        f.write(seg.tobytes())
        print(f"    Wrote segment {idx} with {seg_length} floats ({seg_length*2/1024/1024:.2f} MB)")
        
total_floats = sum(seg.size for seg in segments)
total_bytes = total_floats * 2
print(f"🔹 Total floats exported: {total_floats}")
print(f"🔹 Total binary size: {total_bytes / (1024*1024):.2f} MB")
print(f"✅ Export completed! Weights saved to '{OUTPUT_FILENAME}'.")
