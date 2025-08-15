#!/usr/bin/env python3
import argparse, os, struct, sys, zlib

def read_exact(f, n):
    b=f.read(n); 
    if len(b)!=n: raise EOFError
    return b

def parse_manifest(path):
    with open(path,"rb") as f:
        if read_exact(f,4)!=b"N64W": raise SystemExit("Bad magic")
        ver, align = struct.unpack("<HH", read_exact(f,4))
        if ver not in (1,2): raise SystemExit(f"Bad version: {ver}")
        count, = struct.unpack("<I", read_exact(f,4))
        entries=[]
        for _ in range(count):
            nlen, = struct.unpack("<H", read_exact(f,2))
            name = read_exact(f,nlen).decode("utf-8","replace")
            off, size = struct.unpack("<II", read_exact(f,8))
            crc = struct.unpack("<I", read_exact(f,4))[0] if ver>=2 else None
            entries.append((name, off, size, crc))
        return ver, align, entries

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--bin", required=True)
    ap.add_argument("--man", required=True)
    args=ap.parse_args()

    size=os.path.getsize(args.bin)
    ver, align, entries = parse_manifest(args.man)

    used=[]
    with open(args.bin,"rb") as f:
        for name, off, sz, crc in entries:
            if off%align!=0: print(f"[FAIL] {name}: off {off} not aligned {align}"); sys.exit(2)
            if off+sz>size:  print(f"[FAIL] {name}: OOB ({off}+{sz}>{size})"); sys.exit(2)
            used.append((off, off+sz, name))
            if crc is not None:
                f.seek(off); data = f.read(sz)
                c = zlib.crc32(data) & 0xFFFFFFFF
                if c != crc:
                    print(f"[FAIL] {name}: CRC mismatch exp=0x{crc:08X} got=0x{c:08X}")
                    sys.exit(2)
    used.sort()
    for (s1,e1,n1),(s2,e2,n2) in zip(used,used[1:]):
        if s2<e1: print(f"[FAIL] overlap: {n1}[{s1},{e1}) vs {n2}[{s2},{e2})"); sys.exit(2)

    print(f"[OK] {len(entries)} entries; bin={size} bytes; align={align}; ver={ver}")
if __name__=="__main__": main()
