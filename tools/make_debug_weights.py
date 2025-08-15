#!/usr/bin/env python3
# Produce a small test blob + manifest v2 to exercise ROM streaming/CRC.
import argparse, os, struct, zlib
from pathlib import Path

def align64(f):
    need = (-f.tell()) & 63
    if need: f.write(b"\x00"*need)

ap = argparse.ArgumentParser()
ap.add_argument("--out-dir", default="n64llm/n64-rust/assets")
ap.add_argument("--chunks", type=int, default=8)
ap.add_argument("--chunkKB", type=int, default=64) # 8*64KB = 512KB total default
args = ap.parse_args()

out_dir = Path(args.out_dir); out_dir.mkdir(parents=True, exist_ok=True)
binp = out_dir/"weights.bin"
manp = out_dir/"weights.manifest.bin"

entries=[]
with open(binp,"wb") as f:
    for i in range(args.chunks):
        align64(f)
        off = f.tell()
        data = bytes([(i*31 + j) & 0xFF for j in range(args.chunkKB*1024)])
        f.write(data)
        crc = zlib.crc32(data) & 0xFFFFFFFF
        entries.append((f"debug_chunk_{i}", off, len(data), crc))

with open(manp, "wb") as m:
    m.write(b"N64W")
    m.write(struct.pack("<H", 2)); m.write(struct.pack("<H", 64))
    m.write(struct.pack("<I", len(entries)))
    for name, off, size, crc in entries:
        nb=name.encode("utf-8")
        m.write(struct.pack("<H", len(nb))); m.write(nb)
        m.write(struct.pack("<I", off)); m.write(struct.pack("<I", size))
        m.write(struct.pack("<I", crc))
print(f"[OK] debug weights → {binp} + {manp}")
