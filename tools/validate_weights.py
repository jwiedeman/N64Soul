import argparse, os, struct, sys, zlib

def read_exact(f,n):
    b = f.read(n)
    if len(b)!=n: raise EOFError
    return b

def parse_manifest(path):
    with open(path,"rb") as f:
        if read_exact(f,4) != b"N64W": sys.exit("Bad magic")
        ver, align = struct.unpack("<HH", read_exact(f,4))
        if ver not in (1,2): sys.exit(f"Bad version: {ver}")
        count, = struct.unpack("<I", read_exact(f,4))
        ents=[]
        for _ in range(count):
            nlen, = struct.unpack("<H", read_exact(f,2))
            name = read_exact(f,nlen).decode("utf-8","replace")
            off, size = struct.unpack("<II", read_exact(f,8))
            crc = struct.unpack("<I", read_exact(f,4))[0] if ver==2 else None
            ents.append((name, off, size, crc))
        return ver, align, ents

ap = argparse.ArgumentParser()
ap.add_argument("--bin", required=True)
ap.add_argument("--man", required=True)
ap.add_argument("--crc", action="store_true", help="verify CRC32 for v2")
args = ap.parse_args()

binsz = os.path.getsize(args.bin)
ver, align, ents = parse_manifest(args.man)

used=[]
with open(args.bin,"rb") as f:
    for name, off, sz, crc in ents:
        if off % align: sys.exit(f"[FAIL] {name} off {off} not {align}-aligned")
        if off + sz > binsz: sys.exit(f"[FAIL] {name} OOB {off}+{sz}>{binsz}")
        if args.crc and ver==2:
            f.seek(off); data = f.read(sz)
            c = zlib.crc32(data) & 0xffffffff
            if c != crc: sys.exit(f"[FAIL] {name} CRC mismatch {hex(c)}!={hex(crc)}")
        used.append((off, off+sz, name))
used.sort()
for (s1,e1,n1),(s2,e2,n2) in zip(used, used[1:]):
    if s2 < e1: sys.exit(f"[FAIL] overlap: {n1} [{s1},{e1}) with {n2} [{s2},{e2})")
print(f"[OK] {len(ents)} entries; bin={binsz} align={align} ver={ver}" + (" CRC\u2713" if args.crc and ver==2 else ""))

