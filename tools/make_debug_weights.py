#!/usr/bin/env python3
import os, struct, zlib, argparse
ALIGN=64

ap=argparse.ArgumentParser()
ap.add_argument("--out-bin", required=True)
ap.add_argument("--out-man", required=True)
args=ap.parse_args()
os.makedirs(os.path.dirname(args.out_bin), exist_ok=True)

# two toy entries
entries=[("tok", b"HELLO_DEBUG_TOKENS"), ("ffn", b"\x01\x00\x01\x00")]
with open(args.out_bin,"wb") as b:
    offs=[]
    for name,data in entries:
        pad=(-b.tell()) % ALIGN
        if pad: b.write(b"\x00"*pad)
        off=b.tell(); b.write(data)
        offs.append((name, off, len(data), zlib.crc32(data)&0xffffffff))

with open(args.out_man,"wb") as m:
    m.write(b"N64W"); m.write(struct.pack("<H",2)); m.write(struct.pack("<H",ALIGN))
    m.write(struct.pack("<I", len(offs)))
    for name,off,sz,crc in offs:
        nb=name.encode(); m.write(struct.pack("<H",len(nb))); m.write(nb)
        m.write(struct.pack("<I",off)); m.write(struct.pack("<I",sz)); m.write(struct.pack("<I",crc))
print("debug weights written")

