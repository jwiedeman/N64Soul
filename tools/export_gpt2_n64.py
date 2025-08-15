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

def collect_params_gpt2(model, keep_layers=None):
    # Keep embeddings, final ln_f, lm_head, and a subset of blocks.
    # keep_layers=None keeps all; otherwise keep the last N transformer blocks.
    wanted = set()
    if keep_layers is not None:
        L = len(model.transformer.h)
        keep = set(range(max(0, L - keep_layers), L))
    else:
        keep = None

    for n, p in model.named_parameters():
        if n.startswith("transformer.wte") or n.startswith("transformer.wpe"):
            wanted.add(n)
        elif n.startswith("transformer.ln_f"):
            wanted.add(n)
        elif n.startswith("lm_head"):
            wanted.add(n)
        elif n.startswith("transformer.h."):
            # block index
            try:
                blk = int(n.split(".")[2])
            except Exception:
                continue
            if (keep is None) or (blk in keep):
                wanted.add(n)
    # Deterministic order: embeddings → blocks (by idx) → ln_f → lm_head
    def key(n):
        if n.startswith("transformer.wte"): return (0, 0, n)
        if n.startswith("transformer.wpe"): return (0, 1, n)
        if n.startswith("transformer.h."):
            parts = n.split(".")
            return (1, int(parts[2]), n)
        if n.startswith("transformer.ln_f"): return (2, 0, n)
        if n.startswith("lm_head"):          return (3, 0, n)
        return (9, 0, n)
    return [n for n in sorted(wanted, key=key)]

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
    from transformers import AutoModelForCausalLM
    model = AutoModelForCausalLM.from_pretrained(args.model, torch_dtype=(torch.float16 if args.dtype=="fp16" else torch.float32), low_cpu_mem_usage=True)
    model = model.to("cpu")
    model.eval()

    names = collect_params_gpt2(model, keep_layers=args.keep_layers)

    entries = []
    with open(out_bin, "wb") as bout:
        for name in names:
            tensor = dict(model.named_parameters())[name].detach().cpu()
            if args.dtype == "fp16":
                tensor = tensor.to(torch.float16)
            data = tensor.contiguous().numpy().tobytes()
            align64(bout)
            off = bout.tell()
            bout.write(data)
            crc = zlib.crc32(data) & 0xFFFFFFFF
            entries.append((name, off, len(data), crc))

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
