#!/usr/bin/env python3
# Minimal GPT-2 exporter → n64 format (64B aligned). fp16 by default.
# Requires: torch, transformers
import argparse, os, json, time, struct, zlib
from pathlib import Path

def align64(f):
    need = (-f.tell()) & 63
    if need: f.write(b"\x00"*need)
    return need

def write_manifest_v2(path, align, entries):
    # entries: list of (name:str, off:int, size:int, crc32:int)
    with open(path, "wb") as m:
        m.write(b"N64W")
        m.write(struct.pack("<H", 2))           # ver=2 (CRC aware)
        m.write(struct.pack("<H", align))
        m.write(struct.pack("<I", len(entries)))
        for name, off, size, crc in entries:
            nb = name.encode("utf-8")
            m.write(struct.pack("<H", len(nb)))
            m.write(nb)
            m.write(struct.pack("<I", off))
            m.write(struct.pack("<I", size))
            m.write(struct.pack("<I", crc))

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", default="gpt2", help="HF id or local path (e.g. gpt2-medium)")
    ap.add_argument("--dtype", choices=["fp16","fp32"], default="fp16")
    ap.add_argument("--keep-layers", type=int, default=None, help="Keep last N blocks (prunes depth)")
    ap.add_argument("--out-dir", default="n64llm/n64-rust/assets", help="Output dir for weights.bin/manifest.bin")
    ap.add_argument("--tune-config", default=None, help="JSON file to archive alongside export metadata")
    ap.add_argument("--no-smoke", action="store_true", help="Skip smoke run")
    args = ap.parse_args()

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    out_bin = out_dir / "weights.bin"
    out_man = out_dir / "weights.manifest.bin"

    # --- load model
    import torch
    from transformers import AutoModelForCausalLM, AutoTokenizer
    model = AutoModelForCausalLM.from_pretrained(args.model, torch_dtype=(torch.float16 if args.dtype=="fp16" else torch.float32), low_cpu_mem_usage=True)
    model = model.to("cpu")
    model.eval()

    tokenizer = AutoTokenizer.from_pretrained(args.model)

    def tensor_bytes(t):
        return t.detach().cpu().to(torch.float32).contiguous().numpy().astype("<f4").tobytes()

    entries = []

    def write_blob(name, blob, bout):
        align64(bout)
        off = bout.tell()
        bout.write(blob)
        crc = zlib.crc32(blob) & 0xFFFFFFFF
        entries.append((name, off, len(blob), crc))

    def write_tensor(name, tensor, bout):
        data = tensor_bytes(tensor)
        write_blob(name, data, bout)

    d_model = int(model.config.n_embd)
    all_layers = list(range(len(model.transformer.h)))
    if args.keep_layers is not None:
        all_layers = all_layers[-args.keep_layers:]
    n_layer = len(all_layers)
    n_head = int(model.config.n_head)
    n_positions = int(getattr(model.config, "n_positions", getattr(model.config, "n_ctx", 0)))
    if n_positions == 0:
        n_positions = tokenizer.model_max_length
    d_ff_attr = getattr(model.config, "n_inner", None)
    if d_ff_attr is None:
        d_ff = d_model * 4
    else:
        d_ff = int(d_ff_attr)
    vocab_size = int(getattr(model.config, "vocab_size", tokenizer.vocab_size))

    meta = struct.pack(
        "<IIIIIIII",
        0x4D455441,
        1,
        d_model,
        vocab_size,
        n_layer,
        n_head,
        n_positions,
        d_ff,
    )

    with open(out_bin, "wb") as bout:
        write_blob("model_meta", meta, bout)
        write_tensor("tok_embeddings", model.transformer.wte.weight, bout)
        write_tensor("pos_embeddings", model.transformer.wpe.weight, bout)

        for export_idx, layer_idx in enumerate(all_layers):
            block = model.transformer.h[layer_idx]
            prefix = f"layer{export_idx}"
            write_tensor(f"{prefix}.ln1.weight", block.ln_1.weight, bout)
            write_tensor(f"{prefix}.ln1.bias", block.ln_1.bias, bout)
            write_tensor(f"{prefix}.attn.qkv.weight", block.attn.c_attn.weight, bout)
            write_tensor(f"{prefix}.attn.qkv.bias", block.attn.c_attn.bias, bout)
            write_tensor(f"{prefix}.attn.proj.weight", block.attn.c_proj.weight, bout)
            write_tensor(f"{prefix}.attn.proj.bias", block.attn.c_proj.bias, bout)
            write_tensor(f"{prefix}.ln2.weight", block.ln_2.weight, bout)
            write_tensor(f"{prefix}.ln2.bias", block.ln_2.bias, bout)
            write_tensor(f"{prefix}.ffn.in.weight", block.mlp.c_fc.weight, bout)
            write_tensor(f"{prefix}.ffn.in.bias", block.mlp.c_fc.bias, bout)
            write_tensor(f"{prefix}.ffn.out.weight", block.mlp.c_proj.weight, bout)
            write_tensor(f"{prefix}.ffn.out.bias", block.mlp.c_proj.bias, bout)

        write_tensor("ln_f.weight", model.transformer.ln_f.weight, bout)
        write_tensor("ln_f.bias", model.transformer.ln_f.bias, bout)
        write_tensor("lm_head", model.lm_head.weight, bout)

        vocab = tokenizer.get_vocab()
        vocab_items = [None] * len(vocab)
        for token, idx in vocab.items():
            vocab_items[idx] = token

        merges = []
        try:
            merge_pairs = tokenizer.backend_tokenizer.model.get_merges()
        except AttributeError:
            merge_pairs = []
        for pair in merge_pairs:
            if isinstance(pair, (list, tuple)) and len(pair) == 2:
                left, right = pair
                merged = left + right
                if left in vocab and right in vocab and merged in vocab:
                    merges.append((vocab[left], vocab[right], vocab[merged]))

        tok_blob = bytearray()
        tok_blob.extend(b"BPE1")
        tok_blob.extend(struct.pack("<H", 1))
        tok_blob.extend(struct.pack("<H", 0))
        tok_blob.extend(struct.pack("<I", len(vocab_items)))
        tok_blob.extend(struct.pack("<I", len(merges)))
        for token in vocab_items:
            data = token.encode("utf-8")
            tok_blob.extend(struct.pack("<H", len(data)))
            tok_blob.extend(data)
        for left_id, right_id, result_id in merges:
            tok_blob.extend(struct.pack("<III", left_id, right_id, result_id))

        write_blob("tokenizer.model", tok_blob, bout)

    write_manifest_v2(out_man, 64, entries)

    # archive export metadata (for reproducibility)
    stamp = time.strftime("%Y%m%d-%H%M%S")
    art = Path("artifacts/exports")/stamp
    art.mkdir(parents=True, exist_ok=True)
    meta = {
        "model": args.model,
        "dtype": args.dtype,
        "keep_layers": args.keep_layers,
        "bin": str(out_bin),
        "man": str(out_man),
        "count": len(entries),
        "bytes": sum(e[2] for e in entries),
    }
    if args.tune_config:
        with open(args.tune_config, "r") as f:
            cfg = json.load(f)
        with open(art/"tune.json","w") as f:
            json.dump(cfg, f, indent=2)
        with open(out_dir/"last_export_tune.json","w") as f:
            json.dump(cfg, f, indent=2)
    with open(art/"export.json","w") as f:
        json.dump(meta, f, indent=2)

    print(f"[OK] export complete → {out_bin} + {out_man}")
    print(f"     entries={meta['count']} bytes={meta['bytes']}")
    # Optional: validate layout using existing validator
    # (supports v1 today; we'll update it below to handle v2)
    # End.
if __name__ == "__main__":
    main()
